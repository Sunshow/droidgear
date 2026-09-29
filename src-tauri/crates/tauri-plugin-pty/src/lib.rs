use std::{
    collections::BTreeMap,
    ffi::OsString,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, PtyPair, PtySize};
use tauri::{
    async_runtime::{Mutex, RwLock},
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[derive(Default)]
struct PluginState {
    session_id: AtomicU32,
    sessions: RwLock<BTreeMap<PtyHandler, Arc<Session>>>,
}

struct Session {
    pair: Mutex<PtyPair>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    child_killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    writer: Mutex<Box<dyn std::io::Write + Send>>,
    reader: Mutex<Box<dyn std::io::Read + Send>>,
}

type PtyHandler = u32;

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn spawn<R: Runtime>(
    file: String,
    args: Vec<String>,
    term_name: Option<String>,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
    env: BTreeMap<String, String>,
    encoding: Option<String>,
    handle_flow_control: Option<bool>,
    flow_control_pause: Option<String>,
    flow_control_resume: Option<String>,

    state: tauri::State<'_, PluginState>,
    _app_handle: AppHandle<R>,
) -> Result<PtyHandler, String> {
    // TODO: Support these parameters
    let _ = term_name;
    let _ = encoding;
    let _ = handle_flow_control;
    let _ = flow_control_pause;
    let _ = flow_control_resume;

    let pty_system = native_pty_system();
    // Create PTY, get the writer and reader
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new(file);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.cwd(OsString::from(cwd));
    }
    for (k, v) in env.iter() {
        cmd.env(OsString::from(k), OsString::from(v));
    }
    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let child_killer = child.clone_killer();
    let handler = state.session_id.fetch_add(1, Ordering::Relaxed);

    let pair = Arc::new(Session {
        pair: Mutex::new(pair),
        child: Mutex::new(child),
        child_killer: Mutex::new(child_killer),
        writer: Mutex::new(writer),
        reader: Mutex::new(reader),
    });
    state.sessions.write().await.insert(handler, pair);
    Ok(handler)
}

#[tauri::command]
async fn write(
    pid: PtyHandler,
    data: String,
    state: tauri::State<'_, PluginState>,
) -> Result<(), String> {
    let session = state
        .sessions
        .read()
        .await
        .get(&pid)
        .ok_or("Unavaliable pid")?
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        session
            .writer
            .blocking_lock()
            .write_all(data.as_bytes())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn read(pid: PtyHandler, state: tauri::State<'_, PluginState>) -> Result<Vec<u8>, String> {
    let session = state
        .sessions
        .read()
        .await
        .get(&pid)
        .ok_or("Unavaliable pid")?
        .clone();
    // PTY reads wait for terminal output; they must not occupy async workers.
    tauri::async_runtime::spawn_blocking(move || {
        let mut buf = vec![0u8; 4096];
        let n = session
            .reader
            .blocking_lock()
            .read(&mut buf)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            Err(String::from("EOF"))
        } else {
            buf.truncate(n);
            Ok(buf)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn resize(
    pid: PtyHandler,
    cols: u16,
    rows: u16,
    state: tauri::State<'_, PluginState>,
) -> Result<(), String> {
    let session = state
        .sessions
        .read()
        .await
        .get(&pid)
        .ok_or("Unavaliable pid")?
        .clone();
    session
        .pair
        .lock()
        .await
        .master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn kill(pid: PtyHandler, state: tauri::State<'_, PluginState>) -> Result<(), String> {
    let session = state
        .sessions
        .read()
        .await
        .get(&pid)
        .ok_or("Unavaliable pid")?
        .clone();
    session
        .child_killer
        .lock()
        .await
        .kill()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn exitstatus(pid: PtyHandler, state: tauri::State<'_, PluginState>) -> Result<u32, String> {
    let session = state
        .sessions
        .read()
        .await
        .get(&pid)
        .ok_or("Unavaliable pid")?
        .clone();
    // A shell may run for the entire app session. Waiting for it belongs on
    // the blocking pool, leaving window commands and other async work runnable.
    tauri::async_runtime::spawn_blocking(move || {
        session
            .child
            .blocking_lock()
            .wait()
            .map(|status| status.exit_code())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("pty")
        .invoke_handler(tauri::generate_handler![
            spawn, write, read, resize, kill, exitstatus
        ])
        .setup(|app_handle, _api| {
            app_handle.manage(PluginState::default());
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        sync::{mpsc, Condvar, Mutex as StdMutex},
        time::Duration,
    };

    #[derive(Clone, Debug)]
    struct BlockingIo {
        entered: mpsc::Sender<()>,
        released: Arc<(StdMutex<bool>, Condvar)>,
    }

    impl BlockingIo {
        fn wait_for_release(&self) {
            self.entered.send(()).unwrap();
            let (lock, ready) = &*self.released;
            let guard = lock.lock().unwrap();
            drop(ready.wait_while(guard, |released| !*released).unwrap());
        }

        fn release(&self) {
            let (lock, ready) = &*self.released;
            *lock.lock().unwrap() = true;
            ready.notify_all();
        }
    }

    impl Read for BlockingIo {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            self.wait_for_release();
            buffer[0] = b'x';
            Ok(1)
        }
    }

    impl Write for BlockingIo {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.wait_for_release();
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl ChildKiller for BlockingIo {
        fn kill(&mut self) -> std::io::Result<()> {
            self.release();
            Ok(())
        }

        fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
            Box::new(self.clone())
        }
    }

    impl Child for BlockingIo {
        fn try_wait(&mut self) -> std::io::Result<Option<portable_pty::ExitStatus>> {
            Ok(None)
        }

        fn wait(&mut self) -> std::io::Result<portable_pty::ExitStatus> {
            self.wait_for_release();
            Ok(portable_pty::ExitStatus::with_exit_code(7))
        }

        fn process_id(&self) -> Option<u32> {
            None
        }

        #[cfg(windows)]
        fn as_raw_handle(&self) -> Option<std::os::windows::io::RawHandle> {
            None
        }
    }

    #[test]
    fn terminal_io_does_not_starve_other_async_commands() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let blocking_io = BlockingIo {
            entered: entered_tx,
            released: Arc::new((StdMutex::new(false), Condvar::new())),
        };
        let session = Arc::new(Session {
            pair: Mutex::new(native_pty_system().openpty(PtySize::default()).unwrap()),
            child: Mutex::new(Box::new(blocking_io.clone())),
            child_killer: Mutex::new(Box::new(blocking_io.clone())),
            writer: Mutex::new(Box::new(blocking_io.clone())),
            reader: Mutex::new(Box::new(blocking_io.clone())),
        });
        let app = tauri::test::mock_builder()
            .manage(PluginState {
                sessions: RwLock::new(BTreeMap::from([(0, session)])),
                ..Default::default()
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(3)
            .build()
            .unwrap();

        let handle = app.handle().clone();
        let reading = runtime.spawn(async move { read(0, handle.state()).await });
        let handle = app.handle().clone();
        let waiting = runtime.spawn(async move { exitstatus(0, handle.state()).await });
        let handle = app.handle().clone();
        let writing = runtime.spawn(async move { write(0, "input".into(), handle.state()).await });

        // Wait until all three commands reach the actual blocking operation.
        // A separate OS thread controls release, so a regression cannot hang the test.
        let all_started = (0..3).all(|_| entered_rx.recv_timeout(Duration::from_secs(5)).is_ok());
        let (heartbeat_tx, heartbeat_rx) = mpsc::channel();
        runtime.spawn(async move { heartbeat_tx.send(()).unwrap() });
        let responsive = heartbeat_rx
            .recv_timeout(Duration::from_millis(500))
            .is_ok();

        blocking_io.release();
        assert_eq!(runtime.block_on(reading).unwrap().unwrap(), b"x");
        assert_eq!(runtime.block_on(waiting).unwrap().unwrap(), 7);
        runtime.block_on(writing).unwrap().unwrap();
        assert!(all_started, "terminal operations did not start");
        assert!(responsive, "terminal I/O blocked unrelated async commands");
    }
}
