use std::{time::Duration, thread::sleep};
use std::process::Command;
use std::env::temp_dir;

struct TranscodeTask {
    uid: usize,
    source_path: String,
    destination_path: String,
    encode_string: Vec<String>,
}

impl TranscodeTask {
    /*
     * The message received is in this order/format: "|.|" delimited
     * this|.|denoted|.|a|.|part|.|of|.|the|.|message
     * 
     * task_uid|.|source_path|.|destination_filename|.|encode_options
     * 1|.|\\192.168.2.30\tvshows\South Park\Season 17\South Park - S17E01 - Let Go, Let Gov Bluray-1080p.mkv|.|C:\Users\Alexi Peck\AppData\Local\Temp\South Park - S17E01 - Let Go, Let Gov Bluray-1080p.mkv|.|-c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k -y
     */
    pub fn new(message: String) -> Self {
        let mut message_iter = message.split("|.|");

        let uid = message_iter
        .next()
        .unwrap()
        .parse::<usize>()
        .unwrap();

        let source_path = message_iter
            .next()
            .unwrap()
            .parse::<String>()
            .unwrap();

        let destination_path: String = match temp_dir().join(message_iter.next().unwrap().parse::<String>().unwrap()).into_os_string().into_string() {
            Ok(t) => {
                t
            },
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
        };
        
        let encode_options = message_iter.next().unwrap().parse::<String>().unwrap();
        let encode_options = encode_options.split(' ');

        let mut encode_string: Vec<String> = vec![
            "-i".to_string(),
            source_path.clone(),
        ];
        
        for segment in encode_options {
            encode_string.push(segment.to_string());
        }

        encode_string.push(destination_path.clone());
        
        println!("{:?}", encode_string);
        Self {
            uid,
            source_path,
            destination_path,
            encode_string,
        }
    }

    pub fn run(&mut self) {
        println!(
            "Encoding file \'{}\'",
            self.source_path,
        );

        let _buffer;
        _buffer = Command::new("ffmpeg")
            .args(&self.encode_string.clone())
            .output()
            .unwrap_or_else(|err| {
                println!("Failed to execute ffmpeg process. Err: {}", err);
                panic!();
            });
        //only uncomment if you want disgusting output
        //should be error, but from ffmpeg, stderr mostly consists of stdout information
        println!("{}", String::from_utf8_lossy(&_buffer.stderr).to_string());
    }
}

fn main() {
    let mut current_transcode: TranscodeTask;
    let mut transcode_queue: Vec<TranscodeTask> = Vec::new();
    let mut transcode_running = false;
    
    //inserting dummy tasks
    transcode_queue.push(TranscodeTask::new(String::from(r"1|.|\\192.168.2.30\tvshows\South Park\Season 17\South Park - S17E01 - Let Go, Let Gov Bluray-1080p.mkv|.|C:\Users\Alexi Peck\AppData\Local\Temp\South Park - S17E01 - Let Go, Let Gov Bluray-1080p.mkv|.|-c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k -y")));
    transcode_queue.push(TranscodeTask::new(String::from(r"2|.|\\192.168.2.30\tvshows\South Park\Season 17\South Park - S17E02 - Informative Murder Porn Bluray-1080p.mkv|.|C:\Users\Alexi Peck\AppData\Local\Temp\South Park - S17E01 - Let Go, Let Gov Bluray-1080p.mkv|.|-c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k -y")));
    
    loop {
        if !transcode_running && !transcode_queue.is_empty() {
            match transcode_queue.pop() {
                Some(t) => {
                    current_transcode = t;
                },
                None => {
                    panic!("");
                }
            }

            //mark worker as running a transcode
            transcode_running = true;

            //for network shares
            //check that credentials are valid
            //check whether the source file is available
            //check whether the destination path is available

            //start transcode
            current_transcode.run();
            
            //make it wait for the transcode to complete before doing anything else but taking messages
            //mark it as no longer
            transcode_running = false;//commented out because transcode runs in a new thread
        }
        sleep(Duration::new(1, 0));
        println!("Test");
    }
}
