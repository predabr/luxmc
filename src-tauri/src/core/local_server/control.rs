use super::*;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub(super) struct Endpoint {
    pub port:u16,
    pub password:String,
}

pub(super) fn properties(previous:&str, endpoint:&Endpoint)->String {
    let keys=["enable-rcon","rcon.port","rcon.password","broadcast-rcon-to-ops"];
    let mut text=previous.lines().filter(|line|!keys.contains(&line.trim_start().split(['=',':',' ','\t']).next().unwrap_or_default())).collect::<Vec<_>>().join("\n");
    text.push_str(&format!("\nenable-rcon=true\nrcon.port={}\nrcon.password={}\nbroadcast-rcon-to-ops=false\n",endpoint.port,endpoint.password));
    text
}

async fn send(stream:&mut tokio::net::TcpStream,id:i32,kind:i32,text:&str)->AppResult<()> {
    if text.len()>4096{return Err(invalid("Comando de controle muito grande."));}
    let mut frame=Vec::with_capacity(text.len()+14);
    frame.extend_from_slice(&((text.len()+10) as i32).to_le_bytes());frame.extend_from_slice(&id.to_le_bytes());frame.extend_from_slice(&kind.to_le_bytes());frame.extend_from_slice(text.as_bytes());frame.extend_from_slice(&[0,0]);
    stream.write_all(&frame).await?;stream.flush().await?;Ok(())
}

async fn receive(stream:&mut tokio::net::TcpStream)->AppResult<(i32,i32,Vec<u8>)> {
    let mut size=[0u8;4];stream.read_exact(&mut size).await?;let size=i32::from_le_bytes(size);
    if !(10..=65536).contains(&size){return Err(invalid("Resposta de controle inválida."));}
    let mut data=vec![0;size as usize];stream.read_exact(&mut data).await?;
    if !data.ends_with(&[0,0]){return Err(invalid("Resposta de controle incompleta."));}
    Ok((i32::from_le_bytes(data[..4].try_into().unwrap()),i32::from_le_bytes(data[4..8].try_into().unwrap()),data[8..data.len()-2].to_vec()))
}

async fn execute_stream(mut stream:tokio::net::TcpStream,endpoint:&Endpoint,text:&str)->AppResult<String> {
    tokio::time::timeout(Duration::from_secs(2),async {
        send(&mut stream,1,3,&endpoint.password).await?;
        let mut authenticated=false;
        for _ in 0..3 {let(id,kind,_)=receive(&mut stream).await?;if id==-1{return Err(unavailable("O controle do servidor recusou a autenticação."));}if id==1 && kind==2{authenticated=true;break;}}
        if !authenticated{return Err(unavailable("O controle do servidor não confirmou a autenticação."));}
        send(&mut stream,2,2,text).await?;
        let first=receive(&mut stream).await;
        if text.eq_ignore_ascii_case("stop") && first.is_err(){return Ok(String::new());}
        let(id,_,body)=first?;if id!=2{return Err(invalid("Resposta de outro comando recebida."));}
        let mut body=body;
        while body.len()<256*1024 {
            match tokio::time::timeout(Duration::from_millis(80),receive(&mut stream)).await {
                Ok(Ok((2,_,next)))=>body.extend_from_slice(&next),
                _=>break,
            }
        }
        Ok(String::from_utf8_lossy(&body).into_owned())
    }).await.map_err(|_|unavailable("O controle do servidor não respondeu em dois segundos."))?
}

#[cfg(test)]
pub(super) async fn execute(endpoint:&Endpoint,text:&str)->AppResult<String> {
    let stream=tokio::net::TcpStream::connect(("127.0.0.1",endpoint.port)).await?;
    execute_stream(stream,endpoint,text).await
}

pub(super) async fn command(runtime:&Runtime,input:&mut tokio::process::ChildStdin,text:&str)->AppResult<()> {
    let endpoint=runtime.control.lock().unwrap_or_else(|error|error.into_inner()).clone();
    if let Some(endpoint)=endpoint {
        if let Ok(Ok(stream))=tokio::time::timeout(Duration::from_millis(500),tokio::net::TcpStream::connect(("127.0.0.1",endpoint.port))).await {
            let response=execute_stream(stream,&endpoint,text).await?;
            record_response(runtime,text,&response);
            return Ok(());
        }
    }
    tokio::time::timeout(Duration::from_secs(2),async {input.write_all(format!("{text}\n").as_bytes()).await?;input.flush().await}).await.map_err(|_|unavailable("O console não respondeu."))??;
    Ok(())
}

fn record_response(runtime:&Runtime,command:&str,response:&str) {
    if matches!(command.trim(),"save-all" | "save-all flush") && response.trim_end().ends_with("Saved the game") {
        runtime.state.lock().unwrap_or_else(|error|error.into_inner()).last_saved_at=Some(chrono::Utc::now().to_rfc3339());
    }
    for line in response.lines().filter(|line|!line.trim().is_empty()){runtime.log("response",line);}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_is_confirmed_by_its_authenticated_command_response_on_newer_minecraft() {
        let (runtime,_)=super::super::tests::fixture_runtime("save".into());
        record_response(&runtime,"list","Saved the game");assert!(runtime.snapshot(0).last_saved_at.is_none());
        record_response(&runtime,"save-all flush","Saving the game (this may take a moment!)");assert!(runtime.snapshot(0).last_saved_at.is_none());
        record_response(&runtime,"save-all flush","Saving the game (this may take a moment!)Saved the game");assert!(runtime.snapshot(0).last_saved_at.is_some());
    }

    #[tokio::test]
    async fn authenticated_control_delivers_command_and_returns_response() {
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=listener.local_addr().unwrap().port();
        let server=tokio::spawn(async move {
            let(mut stream,_)=listener.accept().await.unwrap();let(id,kind,text)=receive(&mut stream).await.unwrap();assert_eq!((id,kind),(1,3));assert_eq!(text,b"private-fixture");
            send(&mut stream,1,2,"").await.unwrap();let(id,kind,text)=receive(&mut stream).await.unwrap();assert_eq!((id,kind),(2,2));assert_eq!(text,b"list");send(&mut stream,2,0,"There are 0 players online").await.unwrap();
        });
        let response=execute(&Endpoint{port,password:"private-fixture".into()},"list").await.unwrap();assert_eq!(response,"There are 0 players online");server.await.unwrap();
        let text=properties("enable-rcon=false\nrcon.password=old\nserver-ip=127.0.0.1\n",&Endpoint{port,password:"private-fixture".into()});assert!(text.contains("server-ip=127.0.0.1"));assert!(!text.contains("rcon.password=old"));assert_eq!(text.matches("enable-rcon=").count(),1);
    }
}
