use super::OptionalFile;
pub struct Embedder{}

impl Embedder{
    pub fn new(request: super::StegHideRequest) -> Result<Embedder,String> {
        match request.embedfile {
            OptionalFile::None => Err("Missing Embed file".to_string()),
            OptionalFile::Stdin => {
                    info!("reading secret data from standard input...");
                    Ok(Embedder{})
            },
            OptionalFile::Some(filename) => {
                info!("reading secret file {}",filename);
                Ok(Embedder{})
            }
        }
    }
    pub fn run(self) -> Result<String, String> {
        Ok(String::from("embedder::new() is not implemented"))
    }
}