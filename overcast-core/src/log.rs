use std::{
    fmt::Display,
    io::{BufWriter, Write},
    sync::mpsc,
    thread::{self, JoinHandle}
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Critical = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Critical => "CRITICAL",
        })
    }
}

pub struct LogManager<T: Write + Send> {
    output: BufWriter<T>,
    receiver: mpsc::Receiver<String>,
}

impl<T: Write + Send + 'static> LogManager<T> {
    pub fn new(output: T, max_logged_level: LogLevel) -> (LogManager<T>, Logger) {
        let (sender, receiver) = mpsc::channel();
        (
            LogManager {
                output: BufWriter::new(output),
                receiver,
            },
            Logger {
                sender,
                log_level: max_logged_level,
            }
        )
    }

    /// Starts logging every received message in the output, 
    /// until all  `Logger` are dropped.
    /// 
    /// This will consume the log manager and send it into it's own thread.
    /// The Log Manager will then listen for incoming logs and write them to the output.
    pub fn start(self) -> JoinHandle<()> {
        let LogManager { mut output, receiver } = self;
        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(message) => match output.write(message.as_bytes()) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Log manager error: unable to write to log file: {e}.");
                            println!("Closing log manager...");
                            break;
                        },
                    },
                    Err(_) => break,
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct Logger {
    sender: mpsc::Sender<String>,
    log_level: LogLevel,
}

impl Logger {
    pub fn log(&self, message: &str, level: LogLevel) {
        if level <= self.log_level {
            let log = format!("#{}: {}\n", level, message);
            match self.sender.send(log) {
                Ok(_) => {},
                Err(_) => println!("#{}: {}\n", level, message),
            }
        }
    }
}

#[test]
fn test_log_manager() {

    struct TestOutput {
        output: Vec<u8>,
    }

    impl Drop for TestOutput {
        fn drop(&mut self) {
            // when dropped, test if the messages have been logged
            assert_eq!(
                String::from_utf8(self.output.clone()).unwrap(),
                "#INFO: logger_1 message_1\n#CRITICAL: logger_2 message_1\n#WARNING: logger_3 message_1\n#DEBUG: logger_1 message_2\n".to_string()
            );
        }
    }

    impl Write for TestOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.output.flush()
        }
    }

    let output = TestOutput { output: Vec::new() };
    let (manager, logger_1) = LogManager::new(output, LogLevel::Debug);
    let handle = manager.start();

    {
        let logger_1 = logger_1; // move logger 1 into this scope
        
        let logger_2 = logger_1.clone();
        
        logger_1.log("logger_1 message_1", LogLevel::Info);
        logger_2.log("logger_2 message_1", LogLevel::Critical);
        
        {
            let logger_3 = logger_1.clone();
            logger_3.log("logger_3 message_1", LogLevel::Warning);
            // drop logger 3
        }
        
        logger_1.log("logger_1 message_2", LogLevel::Debug);
        // drop every logger: log manager should stop, log tester drops and checks the result
    }
    // join, to keep the test alive while the thread performs the assertion
    let _ = handle.join().unwrap();
}