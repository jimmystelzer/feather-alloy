use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;
use std::fs;
use std::path::Path;

pub fn serve_static_files() {
    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:1420").unwrap();
        println!("Servidor HTTP rodando em http://localhost:1420");

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();

                let request = String::from_utf8_lossy(&buffer);
                let path = if let Some(line) = request.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1]
                    } else {
                        "/"
                    }
                } else {
                    "/"
                };

                let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
                println!("Servindo arquivos de: {:?}", ui_dir);
                let file_path = if path == "/" {
                    ui_dir.join("index.html")
                } else {
                    ui_dir.join(&path[1..])
                };


                println!("Requisição: {} -> Arquivo: {:?}", path, file_path);

                let (status, content_type, body) = if file_path.exists() && file_path.is_file() {
                    let content = fs::read(&file_path).unwrap_or_default();
                    let content_type = match file_path.extension().and_then(|s| s.to_str()) {
                        Some("html") => "text/html",
                        Some("css") => "text/css",
                        Some("js") => "application/javascript",
                        Some("json") => "application/json",
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        Some("svg") => "image/svg+xml",
                        _ => "application/octet-stream",
                    };
                    println!("✓ 200 OK - Tipo: {}", content_type);
                    ("200 OK", content_type, content)
                } else {
                    println!("✗ 404 NOT FOUND");
                    ("404 NOT FOUND", "text/html", b"<h1>404 Not Found</h1>".to_vec())
                };

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    status,
                    content_type,
                    body.len()
                );

                stream.write_all(response.as_bytes()).unwrap();
                stream.write_all(&body).unwrap();
                stream.flush().unwrap();
            }
        }
    });
}
