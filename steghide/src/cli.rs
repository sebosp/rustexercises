use clap::App;
use super::encryption_algorithm::*;
use super::encryption_mode::*;
use super::OptionalFile;
use super::RequestMode;
use std::path::Path;
use stderrlog;
use std::str::FromStr;
use std::io::{stdin, stdout, Write};

pub const DEFAULT_COMPRESSION:u8 = 9; // Slowest but smallest
pub const DEFAULT_VERBOSITY:i8 = 1; // Info statements

/// `StegHideCommandBuilder` uses parsed Command line arguments to build
/// an steghide request
pub struct StegHideCommandBuilder {
	embedfile: OptionalFile,
	extractfile: OptionalFile,
	coverfile: OptionalFile,
	stegofile: OptionalFile,
    command_mode: Option<super::CommandMode>,
	passphrase: Option<String>,
	compression_level: Option<u8>,
	marker: String,
	nochecksum: bool,
	embed_name: bool,
	enc_algo: EncryptionAlgorithm,
	enc_mode: EncryptionMode,
	radius: u64,
	goal: f64,
	force: bool,
    verbosity: Option<i8>,
    debug_mode: Option<super::DebugMode>,
	check: bool,
	file_list: Vec<String>,
}

// `cli_exit_clap_invalidvalue` Exits the cli executable with a clap invalid value error
fn clap_invalidvalue(error_string: String) -> clap::Error {
    error!("Invalid values supplied.");
    clap::Error {
        message: error_string,
        kind: clap::ErrorKind::InvalidValue,
        info: None,
    }
}

/// `StegHideCommandBuilder` implements the building pattern
impl StegHideCommandBuilder {
    pub fn new() -> StegHideCommandBuilder {
        debug!("Creating new StegHideCommandBuilder from defaults");
        // Defaults:
        StegHideCommandBuilder {
            embedfile: OptionalFile::Stdin,
            extractfile: OptionalFile::Stdin,
            coverfile: OptionalFile::Stdin,
            stegofile: OptionalFile::None,
            command_mode: None,
            passphrase: None,
            compression_level: None,
            marker: String::from(""),
            nochecksum: false,
            embed_name: true,
            enc_algo: EncryptionAlgorithm::new(), // XXX: How does libmcrypt affects the library.
            enc_mode: EncryptionMode::ECB,
            radius: 0u64, // There is no default Radius for all file formats.
            goal: 100.0f64,
            force: false,
            verbosity: None,
            debug_mode: None,
            check: false,
            file_list: vec![],
        }
    }
    // Helper functions
    /// `get_passphrase_from_prompt` needs user to input the password twice
    fn get_passphrase_from_prompt(&self, double_check: bool) -> Result<String,String> {
        trace!("Requesting passphrase from prompt, double_check: {}", double_check);
        let passphrase = rpassword::prompt_password_stderr("Enter passphrase: ").unwrap();
        if double_check {
            let reenter_pass = rpassword::prompt_password_stderr("Re-Enter passphrase: ").unwrap();
            if reenter_pass != passphrase {
                error!("the passphrases do not match");
                return Err("the passphrases do not match".to_string());
            }
        }
        Ok(passphrase)
    }
    // Builder functions
    /// `with_command` is required, returns the Builder for further customizing
    pub fn with_command(mut self, command: String) -> StegHideCommandBuilder {
        debug!("with_command: {}", command);
        self.command_mode = match command.as_ref() {
            "embed" => Some(super::CommandMode::Embed),
            "extract" => Some(super::CommandMode::Extract),
            "info" => Some(super::CommandMode::Info),
            "encinfo" => Some(super::CommandMode::Encinfo),
            "printfreqs" => Some(super::CommandMode::PrintFreqs),
            _ => None,
        };
        self
    }
    /// `with_passphrase` is optional when mode is "encinfo"
    pub fn with_passphrase(&mut self, passphrase: String) {
        debug!("Setting passphrase");
        self.passphrase = Some(passphrase);
    }
    // Could kindof be a macro, the problem is non-Strings, enums and flags....
    // this came from: grep long src/cli.yml|cut -d: -f2|xargs -I{} echo -e '    pub fn with_{}(&mut self, {}: String) {
    //    trace!("with_{}: {ARG-REMOVE}", {});
    //    self.{} = {};
    //    }'|sed 's/ARG-REMOVE//g'
    pub fn with_embedfile(&mut self, embedfile: String) -> Result<(),String> {
        trace!("with_embedfile: {}", embedfile);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:193
            error!("embedfile provided for non embed operation");
            return Err("the argument 'embedfile' can only be used with the 'embed' command mode".to_string())
        }
        if embedfile == "-" {
            self.embedfile = OptionalFile::Stdin;
        } else {
            self.embedfile = OptionalFile::Some(embedfile);
        }
        Ok(())
    }
    pub fn with_extractfile(&mut self, extractfile: String) -> Result<(),String> {
        trace!("with_extractfile: {}", extractfile);
        if self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:224
            error!("extractfile provided for non extract operation");
            return Err("the argument 'extractfile' can only be used with the 'extract' command mode".to_string());
        }
        if extractfile == "-" {
            self.extractfile = OptionalFile::Stdin;
        } else {
            self.extractfile = OptionalFile::Some(extractfile);
        }
        Ok(())
    }
    pub fn with_coverfile(&mut self, coverfile: String) -> Result<(),String> {
        trace!("with_coverfile: {}", coverfile);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:255
            error!("coverfile provided for non embed operation");
            return Err("the argument 'coverfile' can only be used with the 'embed' command mode".to_string());
        }
        if coverfile == "-" {
            self.coverfile = OptionalFile::Stdin;
        } else {
            self.coverfile = OptionalFile::Some(coverfile);
        }
        Ok(())
    }
    pub fn with_stegofile(&mut self, stegofile: String) -> Result<(),String> {
        trace!("with_stegofile: {}", stegofile);
        if self.command_mode != Some(super::CommandMode::Embed) && self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:286
            error!("stegofile provided for non embed or extract operation");
            return Err("the argument 'stegofile' can only be used with the 'embed' or 'extract' command mode".to_string());
        }
        if stegofile == "-" {
            self.stegofile = OptionalFile::Stdin;
        } else {
            self.stegofile = OptionalFile::Some(stegofile);
        }
        Ok(())
    }
    pub fn with_nochecksum(&mut self) -> Result<(),String> {
        trace!("with_nochecksum");
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:339
            error!("nochecksum provided for non embed operation");
            return Err("the argument 'nochecksum' can only be used with the 'embed' command mode".to_string());
        }
        self.nochecksum = true;
        Ok(())
    }
    pub fn with_compress(&mut self, compress: String) -> Result<(),String> {
        trace!("with_compress: {}", compress);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:365
            error!("compress provided for non embed operation");
            return Err("the arguments 'compress' and 'dontcompress' can only be used with the 'embed' command mode".to_string());
        }
        if self.compression_level.is_none(){
            debug!("Set compression level to '{}' from compress cli argument",compress);
            self.compression_level = Some(compress.parse::<u8>().map_err(|err| format!("Unable to parse compression level: {}: '{}'",err.to_string(),compress))?);
            Ok(())
        } else {
            error!("compression has been already set");
            Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string())
        }
    }
    pub fn with_dontcompress(&mut self) -> Result<(),String> {
        trace!("with_dontcompress");
        self.with_compress("0".to_string()) // Arguments.cc:388
    }
    pub fn with_dontembedname(&mut self) -> Result<(),String> {
        trace!("with_dontembedname");
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:410
            error!("dontembedname provided for non embed operation");
            return Err("the argument 'dontembedname' can only be used with the 'embed' command mode".to_string());
        }
        self.embed_name = false;
        Ok(())
    }
    pub fn with_radius(&mut self, radius: String) -> Result<(),String> {
        trace!("with_radius: {}", radius);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:530
            error!("radius provided for non embed operation");
            return Err("the argument 'radius' can only be used with the 'embed' command mode".to_string());
        }
        self.radius = radius.parse::<u64>().map_err(|err| format!("Unable to parse radius: {}: '{}'",err.to_string(),radius))?;
        Ok(())
    }
    pub fn with_goal(&mut self, goal: String) -> Result<(),String> {
        trace!("with_goal: {}", goal);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:558
            error!("goal provided for non embed operation");
            return Err("the argument 'goal' can only be used with the 'embed' command mode".to_string());
        }
        let goal = goal.parse::<f64>().map_err(|err| format!("Unable to parse goal: {}: '{}'",err.to_string(),goal))?;
        if goal < 0f64 || goal > 100f64 {
            return Err("the argument 'goal' must be followed by a number between 0 and 100".to_string());
        }
        self.goal = goal;
        Ok(())
    }
    pub fn with_marker(&mut self, marker: String) -> Result<(),String> {
        trace!("with_marker: {}", marker);
        if self.command_mode != Some(super::CommandMode::Embed) && self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:589
            error!("marker provided for non embed or extract operation");
            return Err("the argument 'marker' can only be used with the 'embed' or 'extract' command mode".to_string());
        }
        self.marker = marker;
        Ok(())
    }
    pub fn with_force(&mut self) -> Result<(),String> {
        trace!("with_force");
        if self.command_mode != Some(super::CommandMode::Embed) && self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:617
            error!("force provided for non embed or extract operation");
            return Err("the argument 'force' can only be used with the 'embed' or 'extract' command mode".to_string());
        }
        self.force = true;
        Ok(())
    }
    pub fn with_quiet(&mut self) -> Result<(),String> {
        trace!("with_quiet");
        if self.command_mode != Some(super::CommandMode::Embed)
            && self.command_mode != Some(super::CommandMode::Extract)
            && self.command_mode != Some(super::CommandMode::Info)
            { // Arguments.cc:643
            error!("quiet provided for non embed, extract or info operation");
            return Err("the argument 'quiet' can only be used with the 'embed', 'extract' or 'info' command mode".to_string());
        }
        if self.verbosity.is_some() {
            return Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string());
        }
        self.verbosity = Some(0i8);
        Ok(())
    }
    pub fn with_verbosity(&mut self, verbosity: String) -> Result<(),String> {
        trace!("with_verbosity: {}", verbosity);
        if self.command_mode != Some(super::CommandMode::Embed)
            && self.command_mode != Some(super::CommandMode::Extract)
            { // Arguments.cc:657
            error!("verbosity provided for non embed or extract operation");
            return Err("the argument 'verbosity' can only be used with the 'embed' or 'extract' command mode".to_string());
        }
        if self.verbosity.is_some() {
            return Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string());
        }
        self.verbosity = Some(verbosity.parse::<i8>().map_err(|err| format!("Unable to parse verbosity level: {}: '{}'",err.to_string(),verbosity))?);
        Ok(())
    }
    pub fn set_debug(&mut self, mode: super::DebugMode) -> Result<(),String> {
        match self.debug_mode {
            None => self.debug_mode = Some(mode),
            Some(_) => {
                return Err("debug can only have one mode".to_string());
            }
        };
        Ok(())
    }
    pub fn with_debug_printgraph(&mut self) -> Result<(),String> {
        trace!("with_debug_printgraph"); // Arguments.cc:679
        self.set_debug(super::DebugMode::PrintGraph)
    }
    pub fn with_debug_printgmlgraph(&mut self) -> Result<(),String> {
        trace!("with_debug_printgmlgraph"); // Arguments.cc:689
        self.set_debug(super::DebugMode::PrintGmlGraph)
    }
    pub fn with_debug_printgmlvertex(&mut self, printgmlvertex: String) -> Result<(),String> {
        trace!("with_debug_printgmlvertex: {}", printgmlvertex); // Arguments.cc:699
        let vertex_parts:Vec<&str> = printgmlvertex.split(",").collect();
        if vertex_parts.len() != 2 {
            return Err("Invalid parameter for printgmlvertex, expected 'rec_depth,start_vertex' where rec_depth and start_vertex are numbers".to_string());
        }
        let rec_depth = vertex_parts[0].parse::<u64>().map_err(|err| format!("Unable to parse rec_depth: {}: '{}'",err.to_string(),vertex_parts[0]))?;
        let start_vertex = vertex_parts[1].parse::<u64>().map_err(|err| format!("Unable to parse start_vertex: {}: '{}'",err.to_string(),vertex_parts[1]))?;
        trace!("with_debug_printgmlvertex: parsed to DebugMode::PrintGmlVertex({},{})", rec_depth,start_vertex);
        self.set_debug(super::DebugMode::PrintGmlVertex(rec_depth,start_vertex))
    }
    pub fn with_debug_printstats(&mut self) -> Result<(),String> {
        trace!("with_debug_printstatsq"); // Arguments.cc:717
        self.set_debug(super::DebugMode::PrintStats)
    }
    pub fn with_debuglevel(&mut self, debuglevel: String) -> Result<(),String> {
        trace!("with_debuglevel: {}", debuglevel); // Arguments.cc:731
        let debug_level = debuglevel.parse::<u64>().map_err(|err| format!("Unable to parse debug_level: {}: '{}'",err.to_string(),debuglevel))?;
        // XXX: Check what is the maximum debug_level
        self.set_debug(super::DebugMode::DebugLevel(debug_level))
    }
    pub fn with_check(&mut self) -> Result<(),String> {
        trace!("with_check"); // Arguments.cc:747
        self.set_debug(super::DebugMode::Check)
    }
    pub fn with_encryption(&mut self, encryption: String) -> Result<(),String> {
        trace!("with_encryption: {}", encryption);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:433
            error!("encryption provided for non embed operation");
            return Err("the argument 'encryption' can only be used with the 'embed' command mode".to_string());
        }
        return Err("--encryption is not yet supported".to_string());
    }
    // Validation related functions:
    /// `policy_set_stegofile_from_cover` sets stegofile to be the cover file
    /// when stegofile is not set and coverfile is set.
    fn policy_set_stegofile_from_cover(&mut self){
        self.stegofile = match &self.coverfile {
            &OptionalFile::Some(ref coverfile) => OptionalFile::Some(coverfile.clone()),
            &OptionalFile::None => OptionalFile::None,
            &OptionalFile::Stdin => OptionalFile::Stdin,
        };
        self.force = true;
    }
    /// `validate_build` checks the arguments passed are valid before returning the StegHide request.
    fn validate_build(&mut self) -> Result<(),String> {
        debug!("Validating build");
        if self.command_mode == Some(super::CommandMode::Embed) { // Arguments.cc:97
            if self.coverfile.is_stdin() && self.embedfile.is_stdin() {
                return Err("standard input cannot be used for cover data AND data to be embedded".to_string());
            }
            if self.stegofile.is_none() && self.coverfile.is_some() {
                debug!("Stegofile is not set, copying coverfile value");
                self.policy_set_stegofile_from_cover();
            }
        }
        if self.command_mode == Some(super::CommandMode::Embed) || self.command_mode == Some(super::CommandMode::Extract) { // Arguments.cc:107
            debug!("Checking passphrase");
            if self.passphrase.is_none(){
                trace!("Password is not set, requesting");
                let mut double_check_passphrase = false;
                // prompt for passphrase
                if self.command_mode == Some(super::CommandMode::Embed){
                    if self.coverfile.is_stdin() || self.embedfile.is_stdin() {
                        return Err("if standard input is used, the passphrase must be specified on the command line".to_string());
                    }
                    // When embedding, the passphrase should be requested twice
                    double_check_passphrase = true;
                    self.passphrase = Some(self.get_passphrase_from_prompt(double_check_passphrase).unwrap());
                } else { // Extract
                    if self.stegofile.is_none() {
                        return Err("if standard input is used, the passphrase must be specified on the command line".to_string());
                    }
                    self.passphrase = Some(self.get_passphrase_from_prompt(double_check_passphrase).unwrap());
                }
            }
        }
        if self.compression_level.is_none(){
            debug!("Setting compression level to default");
            self.compression_level = Some(DEFAULT_COMPRESSION);
        }
        if self.verbosity.is_none(){
            debug!("Setting verbosity level to default");
            self.verbosity = Some(DEFAULT_VERBOSITY);
        }
        if self.command_mode == Some(super::CommandMode::Extract) {
            if let OptionalFile::Some(extract_filename) = &self.extractfile {
                if Path::new(&extract_filename).exists() && ! self.force {
                    let user_question = format!("the file \"{}\" does already exist. overwrite ?",extract_filename);
                    if !self.request_user_bool_response(user_question) {
                        return Err("Overwrite cancelled".to_string())
                    }
                }
            }
        }
        Ok(())
    }
    pub fn request_user_bool_response(&self, question: String) -> bool {
        info!("Requesting user input for question '{}'", question);
        print!("{}", question);
        stdout().flush().unwrap();
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        if buffer == "y\n" || buffer == "Y\n" {
            trace!("Got true from user input");
            true
        } else {
            trace!("Got false from user input");
            false
        }
    }
    /// `build` finishes the builder for StepHideRequest after validating parameters
    pub fn build(mut self) -> Result<super::StegHideRequest, String> {
        match self.validate_build() {
            Ok(()) => trace!("SteghideRequestBuilder validation complete"),
            Err(err) => clap_invalidvalue(err).exit(),
        };
        Ok(super::StegHideRequest{
            passphrase: self.passphrase.unwrap(),
            compression_level: self.compression_level.unwrap(),
            command: self.command_mode.unwrap(),
            embedfile: self.embedfile,
            extractfile: self.extractfile,
            coverfile: self.coverfile,
            stegofile: self.stegofile,
            marker: self.marker,
            nochecksum: self.nochecksum,
            embed_name: self.embed_name,
            enc_algo: self.enc_algo,
            enc_mode: self.enc_mode,
            radius: self.radius,
            goal: self.goal,
            force: self.force,
            verbosity: self.verbosity.unwrap(),
            debug_mode: self.debug_mode,
            check: self.check,
            file_list: self.file_list,
            request_mode: RequestMode::CommandLine,
        })
    }
}
pub fn parse_optional_args(builder: &mut StegHideCommandBuilder, clap_args: &clap::ArgMatches<'_>) -> Result<(),String> {
    let cli_yaml = load_yaml!("cli.yml");
    let clap_args_copy = App::from_yaml(cli_yaml).get_matches();
    for (arg, _) in clap_args_copy.args {
        match arg.as_ref() {
            "passphrase" => {
                builder.with_passphrase(clap_args.value_of("passphrase").unwrap().to_string());
            },
            "embedfile" => {
                builder.with_embedfile(clap_args.value_of("embedfile").unwrap().to_string())?;
            },
            "extractfile" => {
                builder.with_extractfile(clap_args.value_of("extractfile").unwrap().to_string())?;
            },
            "coverfile" => {
                builder.with_coverfile(clap_args.value_of("coverfile").unwrap().to_string())?;
            },
            "stegofile" => {
                builder.with_stegofile(clap_args.value_of("stegofile").unwrap().to_string())?;
            },
            "nochecksum" => {
                builder.with_nochecksum()?;
            },
            "compress" => {
                builder.with_compress(clap_args.value_of("compress").unwrap().to_string())?;
            },
            "dontcompress" => {
                builder.with_dontcompress()?;
            },
            "dontembedname" => {
                builder.with_dontembedname()?;
            },
            "radius" => {
                builder.with_radius(clap_args.value_of("radius").unwrap().to_string())?;
            },
            "goal" => {
                builder.with_goal(clap_args.value_of("goal").unwrap().to_string())?;
            },
            "marker" => {
                builder.with_marker(clap_args.value_of("marker").unwrap().to_string())?;
            },
            "force" => {
                builder.with_force()?;
            },
            "quiet" => {
                builder.with_quiet()?;
            },
            "verbosity" => {
                builder.with_verbosity(clap_args.value_of("verbosity").unwrap().to_string())?;
            },
            "printgraph" => {
                builder.with_debug_printgraph()?;
            },
            "printgmlgraph" => {
                builder.with_debug_printgmlgraph()?;
            },
            "printgmlvertex" => {
                builder.with_debug_printgmlvertex(clap_args.value_of("printgmlvertex").unwrap().to_string())?;
            },
            "debuglevel" => {
                builder.with_debuglevel(clap_args.value_of("debuglevel").unwrap().to_string())?;
            },
            "printstats" => {
                builder.with_debug_printstats()?;
            },
            "check" => {
                builder.with_check()?;
            },
            "encryption" => {
                builder.with_encryption(clap_args.value_of("encryption").unwrap().to_string())?;
            },
            &_ => {}, // Handle MODE, version, help, author, etc
        };
    }
    Ok(())
}
/// `parse_arguments` parses command line flags, sets up logging and timestamp
///  format
pub fn parse_arguments() -> Result<super::StegHideRequest,String> {
    // Parse command line arguments from cli.yml
    let cli_yaml = load_yaml!("cli.yml");
    let clap_args = App::from_yaml(cli_yaml).get_matches();
    let verbosity = clap_args.value_of("verbosity").map(|v| {
        v.parse::<usize>().unwrap_or_else(|_| {
            clap_invalidvalue(
                "invalid value for 'verbosity', should be a value from 0 to 4".to_string()
            ).exit()
        })
    }).unwrap_or(DEFAULT_VERBOSITY as usize);
    let quiet = clap_args.is_present("quiet");
    let ts = clap_args.value_of("timestamp").map(|v| {
        stderrlog::Timestamp::from_str(v).unwrap_or_else(|_| {
            clap_invalidvalue(
                "invalid value for 'timestamp'".to_string()
            ).exit()
        })
    }).unwrap_or(stderrlog::Timestamp::Off);
    // Initialize logger
    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbosity)
        .timestamp(ts)
        .init()
        .unwrap();
    trace!("Starting CLI arg validation");
    let command_mode = clap_args.value_of("MODE").unwrap();
    let mut builder = StegHideCommandBuilder::new()
        .with_command(command_mode.to_string());
    match parse_optional_args(&mut builder, &clap_args) {
        Err(error_string) => clap_invalidvalue(error_string.clone()).exit(),
        Ok(()) => debug!("Optional Arguments provided correctly"),
    };
    if ! builder.request_user_bool_response(format!("Yo, enter Y: ")) {
        return Err("Got false on 'Y' request_user_bool_response".to_string());
    }
    if ! builder.request_user_bool_response(format!("Yo, enter y: ")) {
        return Err("Got false on 'Y' request_user_bool_response".to_string());
    }
    if builder.request_user_bool_response(format!("Yo, enter N: ")) {
        return Err("Got true on 'N' request_user_bool_response".to_string());
    }
    builder.build()
}

/// `run_from_arguments` is the main entrypoint for CLI request
/// XXX: Returns Ok(Statistics)
pub fn run_from_arguments() -> Result<(),String> {
    let request = parse_arguments()?;
    match request.run(){
        Ok(_) => Ok(()),
        Err(err) => Err(err)
    }
}
#[cfg(test)]
mod tests {
    use super::StegHideCommandBuilder;
    use super::OptionalFile;
    #[test]
    fn it_checks_with_command() {
        let good_builder = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(good_builder.command_mode.is_some(),true);
        let bad_builder = StegHideCommandBuilder::new()
            .with_command("NOCOMMAND".to_string());
        assert_eq!(bad_builder.command_mode.is_none(),true);
    }
    #[test]
    fn it_validates_invalid_stdin_both_cover_embed() {
        let mut builder = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        builder.with_passphrase("aoeu".to_string());
        assert_eq!(builder.with_coverfile("-".to_string()),Ok(()));
        assert_eq!(builder.with_embedfile("-".to_string()), Ok(()));
        assert_eq!(builder.validate_build(), Err("standard input cannot be used for cover data AND data to be embedded".to_string()));
    }
    #[test]
    fn it_sets_stegofile_from_cover_when_empty() {
        let mut builder = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        builder.with_passphrase("aoeu".to_string());
        assert_eq!(builder.with_coverfile("a".to_string()),Ok(()));
        assert_eq!(builder.with_embedfile("-".to_string()),Ok(()));
        assert_eq!(builder.validate_build(), Ok(()));
        assert_eq!(builder.stegofile, builder.coverfile);
    }
    #[test]
    fn it_checks_passphrase_argument_needed_on_embed() {
        let mut cover_none = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(cover_none.with_embedfile("a".to_string()),Ok(()));
        assert_eq!(
            cover_none.validate_build(),
            Err("if standard input is used, the passphrase must be specified on the command line".to_string())
        );
        let mut embed_none = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_none.with_coverfile("a".to_string()),Ok(()));
        assert_eq!(
            embed_none.validate_build(),
            Err("if standard input is used, the passphrase must be specified on the command line".to_string())
        );
    }
    #[test]
    fn it_checks_passphrase_argument_needed_on_extract() {
        let mut cover_none = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(cover_none.with_extractfile("a".to_string()),Ok(()));
        assert_eq!(
            cover_none.validate_build(),
            Err("if standard input is used, the passphrase must be specified on the command line".to_string())
        );
        let mut embed_none = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(
            embed_none.validate_build(),
            Err("if standard input is used, the passphrase must be specified on the command line".to_string())
        );
    }
    #[test]
    fn it_checks_embedfile_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_embedfile("test".to_string()),Ok(()));
        assert_eq!(embed_mode.embedfile, OptionalFile::Some("test".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_embedfile("test".to_string()),Err("the argument 'embedfile' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_checks_extractfile_needs_extract_mode() {
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_extractfile("test".to_string()),Ok(()));
        assert_eq!(extract_mode.extractfile, OptionalFile::Some("test".to_string()));
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_extractfile("test".to_string()),Err("the argument 'extractfile' can only be used with the 'extract' command mode".to_string()));
    }
    
    #[test]
    fn it_checks_coverfile_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_coverfile("test".to_string()),Ok(()));
        assert_eq!(embed_mode.coverfile, OptionalFile::Some("test".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_coverfile("test".to_string()),Err("the argument 'coverfile' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_checks_stegofile_needs_embed_or_extract_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_stegofile("test".to_string()),Ok(()));
        assert_eq!(embed_mode.stegofile, OptionalFile::Some("test".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_stegofile("test".to_string()),Ok(()));
        assert_eq!(extract_mode.stegofile, OptionalFile::Some("test".to_string()));
        let mut encinfo_mode = StegHideCommandBuilder::new()
            .with_command("encinfo".to_string());
        assert_eq!(encinfo_mode.with_stegofile("test".to_string()),Err("the argument 'stegofile' can only be used with the 'embed' or 'extract' command mode".to_string()
));
    }
    #[test]
    fn it_checks_nochecksum_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.nochecksum, false); // XXX: this should be set as a default state
        assert_eq!(embed_mode.with_nochecksum(),Ok(()));
        assert_eq!(embed_mode.nochecksum, true);
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_nochecksum(),Err("the argument 'nochecksum' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_checks_compress_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        //assert_eq!(embed_mode.compression_level, Some(DEFAULT_COMPRESSION)); XXX: create an object complete enough to validate this
        assert_eq!(embed_mode.with_compress("a".to_string()),Err("Unable to parse compression level: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_compress("1".to_string()),Ok(()));
        assert_eq!(embed_mode.compression_level, Some(1u8));
        // Compress can only be provided once, but dontcompress would also target the same field in the struct
        assert_eq!(embed_mode.with_dontcompress(),Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string()));
        // Calling the same function twice returns the same error message, but clap forbids us from using --compress twice
        assert_eq!(embed_mode.with_compress("1".to_string()),Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_compress("1".to_string()),Err("the arguments 'compress' and 'dontcompress' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_checks_dontcompress_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        //assert_eq!(embed_mode.compression_level, Some(DEFAULT_COMPRESSION)); XXX: create an object complete enough to validate this
        assert_eq!(embed_mode.with_dontcompress(),Ok(()));
        assert_eq!(embed_mode.compression_level, Some(0u8));
        // Compress can only be provided once, but dontcompress would also target the same field in the struct
        assert_eq!(embed_mode.with_dontcompress(),Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string()));
        // Calling the same function twice returns the same error message, but clap forbids us from using --compress twice
        assert_eq!(embed_mode.with_compress("1".to_string()),Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_dontcompress(),Err("the arguments 'compress' and 'dontcompress' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_checks_dontembedname_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_dontembedname(),Ok(()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_dontembedname(),Err("the argument 'dontembedname' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_handles_radius_input() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_radius("a".to_string()),Err("Unable to parse radius: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_radius("1".to_string()),Ok(()));
        assert_eq!(embed_mode.radius, 1u64);
        assert_eq!(embed_mode.with_radius("-1.1".to_string()),Err("Unable to parse radius: invalid digit found in string: '-1.1'".to_string()));
        assert_eq!(embed_mode.with_radius("1.0".to_string()),Err("Unable to parse radius: invalid digit found in string: '1.0'".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_radius("1".to_string()),Err("the argument 'radius' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_handles_goal_input() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_goal("a".to_string()),Err("Unable to parse goal: invalid float literal: 'a'".to_string()));
        assert_eq!(embed_mode.with_goal("1".to_string()),Ok(()));
        assert_eq!(embed_mode.with_goal("1.0".to_string()),Ok(()));
        assert_eq!(embed_mode.with_goal("-1.0".to_string()),Err("the argument 'goal' must be followed by a number between 0 and 100".to_string()));
        assert_eq!(embed_mode.with_goal("101.0".to_string()),Err("the argument 'goal' must be followed by a number between 0 and 100".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_goal("1".to_string()),Err("the argument 'goal' can only be used with the 'embed' command mode".to_string()));
    }
    #[test]
    fn it_handles_conflicting_debug_input() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_debuglevel("1".to_string()),Ok(()));
        assert_eq!(embed_mode.with_debug_printgmlgraph(),Err("debug can only have one mode".to_string()));
        assert_eq!(embed_mode.with_debug_printgmlvertex("1,1".to_string()),Err("debug can only have one mode".to_string()));
        assert_eq!(embed_mode.with_debug_printgraph(),Err("debug can only have one mode".to_string()));
        assert_eq!(embed_mode.with_debug_printstats(),Err("debug can only have one mode".to_string()));
    }
    #[test]
    fn it_handles_debug_printgmlvertex_input() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_debug_printgmlvertex("1,a".to_string()),Err("Unable to parse start_vertex: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_debug_printgmlvertex("a,1".to_string()),Err("Unable to parse rec_depth: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_debug_printgmlvertex("1,2,3".to_string()),Err("Invalid parameter for printgmlvertex, expected 'rec_depth,start_vertex' where rec_depth and start_vertex are numbers".to_string()));
        assert_eq!(embed_mode.with_debug_printgmlvertex("1".to_string()),Err("Invalid parameter for printgmlvertex, expected 'rec_depth,start_vertex' where rec_depth and start_vertex are numbers".to_string()));
        assert_eq!(embed_mode.with_debug_printgmlvertex("1,2".to_string()),Ok(()));
        assert_eq!(embed_mode.debug_mode,Some(super::super::DebugMode::PrintGmlVertex(1u64,2u64)));
    }
    #[test]
    fn it_checks_marker_needs_embed_or_extract_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_marker("test".to_string()),Ok(()));
        assert_eq!(embed_mode.marker, "test".to_string());
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_marker("test".to_string()),Ok(()));
        assert_eq!(extract_mode.marker, "test".to_string());
        let mut encinfo_mode = StegHideCommandBuilder::new()
            .with_command("encinfo".to_string());
        assert_eq!(encinfo_mode.with_marker("test".to_string()),Err("the argument 'marker' can only be used with the 'embed' or 'extract' command mode".to_string()
));
    }
    #[test]
    fn it_checks_force_needs_embed_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.force, false);
        assert_eq!(embed_mode.with_force(),Ok(()));
        assert_eq!(embed_mode.force, true);
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.force, false);
        assert_eq!(extract_mode.with_force(),Ok(()));
        assert_eq!(extract_mode.force, true);
        let mut encinfo_mode = StegHideCommandBuilder::new()
            .with_command("encinfo".to_string());
        assert_eq!(encinfo_mode.with_force(),Err("the argument 'force' can only be used with the 'embed' or 'extract' command mode".to_string()
));
    }
    #[test]
    fn it_checks_verbosity_needs_embed_or_extract_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_verbosity("a".to_string()),Err("Unable to parse verbosity level: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_verbosity("3".to_string()),Ok(()));
        assert_eq!(embed_mode.verbosity, Some(3i8));
        assert_eq!(embed_mode.with_quiet(),Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_verbosity("3".to_string()),Ok(()));
        assert_eq!(extract_mode.verbosity, Some(3i8));
        assert_eq!(extract_mode.with_quiet(),Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string()));
        let mut encinfo_mode = StegHideCommandBuilder::new()
            .with_command("encinfo".to_string());
        assert_eq!(encinfo_mode.with_verbosity("3".to_string()),Err("the argument 'verbosity' can only be used with the 'embed' or 'extract' command mode".to_string()
));
    }
    #[test]
    fn it_checks_quiet_needs_embed_extract_or_info_mode() {
        let mut embed_mode = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        assert_eq!(embed_mode.with_verbosity("a".to_string()),Err("Unable to parse verbosity level: invalid digit found in string: 'a'".to_string()));
        assert_eq!(embed_mode.with_quiet(),Ok(()));
        assert_eq!(embed_mode.verbosity, Some(0i8));
        assert_eq!(embed_mode.with_verbosity("3".to_string()),Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string()));
        let mut extract_mode = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        assert_eq!(extract_mode.with_quiet(),Ok(()));
        assert_eq!(extract_mode.verbosity, Some(0i8));
        assert_eq!(extract_mode.with_verbosity("3".to_string()),Err("the 'verbosity' argument conflicts with the 'quiet' argument".to_string()));
        let mut info_mode = StegHideCommandBuilder::new()
            .with_command("info".to_string());
        assert_eq!(info_mode.with_quiet(),Ok(()));
        assert_eq!(info_mode.verbosity, Some(0i8));
        assert_eq!(info_mode.with_verbosity("3".to_string()),Err("the argument 'verbosity' can only be used with the 'embed' or 'extract' command mode".to_string()));
        let mut encinfo_mode = StegHideCommandBuilder::new()
            .with_command("encinfo".to_string());
        assert_eq!(encinfo_mode.with_quiet(),Err("the argument 'quiet' can only be used with the 'embed', 'extract' or 'info' command mode".to_string()
));
    }
}
