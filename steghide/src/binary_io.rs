use std::fs;
use steghide::OptionalFile;
use std::path::Path;

enum OpenMode {
    Read,
    Write
}
pub struct BinaryIO {
    mode: OpenMode,
    name: String,
    // data: BufReader,
}
impl BinaryIO {
    fn open(self, file: OptionalFile, mode: OpenMode) {
        if file.is_stdin() {
            match mode {
                OpenMode::Read => unimplemented!(),
                OpenMode::Write => unimplemented!(),
            };
        }
    }
    pub fn check_force(self, request: StegHideRequest) -> bool {
        if !request.force {
            if Path::new(self.name).exists(){
                // If Request is CLI, the user has already been asked to double-check.
                if request.request_mode::HTTPRequest {
                    return false;
                }
                // If Request is HTTP, fail the request
            }
        }
    }
}