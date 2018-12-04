use clap::App;
use super::encryption_algorithm::*;
use super::encryption_mode::*;
use stderrlog;
use std::str::FromStr;
use rpassword::read_password;

pub const DEFAULT_COMPRESSION:u8 = 9; // Slowest but smallest

/// `StegHideCommandBuilder` uses parsed Command line arguments to build
/// an steghide request
pub struct StegHideCommandBuilder {
	// embed file name, "-" if stdin
	embedfile: Option<String>,
	// extract file name, "-" if stdout
	extractfile: Option<String>,
	// cover file name, "-" if stdin
	coverfile: Option<String>,
	// stego file name, "-" if stdout/stdin
	stegofile: Option<String>,
    // Info file name, "-"
	infofile: Option<String>,
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
    verbosity: i8,
    debug_mode: Option<super::DebugMode>,
	check: bool,
	file_list: Vec<String>,
	gml_graph_rec_depth: u64,
	gml_start_vertex: u64,
}

/// `StegHideCommandBuilder` implements the building pattern
impl StegHideCommandBuilder {
    pub fn new() -> StegHideCommandBuilder {
        debug!("Creating new StegHideCommandBuilder from defaults");
        // Defaults:
        StegHideCommandBuilder {
            embedfile: None,
            extractfile: None,
            coverfile: None,
            stegofile: None,
            infofile: None,
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
            verbosity: 1i8, // XXX: Check out python logging equivalent for Rust.
            debug_mode: None,
            check: false,
            file_list: vec![String::from("")],
            gml_graph_rec_depth: 0u64,
            gml_start_vertex: 0u64,
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
    fn cli_exit_clap_invalidvalue(&self, error_string: String) {
        error!("Invalid values. Exiting.");
        clap::Error {
            message: error_string,
            kind: clap::ErrorKind::InvalidValue,
            info: None,
        }.exit();
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
        self.embedfile = Some(embedfile);
        Ok(())
    }
    pub fn with_extractfile(&mut self, extractfile: String) -> Result<(),String> {
        trace!("with_extractfile: {}", extractfile);
        if self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:224
            error!("extractfile provided for non extract operation");
            return Err("the argument 'extractfile' can only be used with the 'extract' command mode".to_string());
        }
        self.extractfile = Some(extractfile);
        Ok(())
    }
    pub fn with_coverfile(&mut self, coverfile: String) -> Result<(),String> {
        trace!("with_coverfile: {}", coverfile);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:255
            error!("coverfile provided for non embed operation");
            return Err("the argument 'coverfile' can only be used with the 'embed' command mode".to_string());
        }
        self.coverfile = Some(coverfile);
        Ok(())
    }
    pub fn with_stegofile(&mut self, stegofile: String) -> Result<(),String> {
        trace!("with_stegofile: {}", stegofile);
        if self.command_mode != Some(super::CommandMode::Embed) && self.command_mode != Some(super::CommandMode::Extract) { // Arguments.cc:286
            error!("stegofile provided for non embed or extract operation");
            return Err("the argument 'stegofile' can only be used with the 'embed' or 'extract' command mode".to_string());
        }
        self.stegofile = Some(stegofile);
        Ok(())
    }
    pub fn with_infofile(&mut self, infofile: String) -> Result<(),String> {
        trace!("with_infofile: {}", infofile);
        self.infofile = Some(infofile);
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
            return Err("the argument 'compress' can only be used with the 'embed' command mode".to_string());
        }
        if self.compression_level.is_none(){
            debug!("Set compression level to 0 from compress cli argument");
            self.compression_level = Some(compress.parse::<u8>().unwrap());
        } else {
            error!("compression has been already set");
            return Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string());
        }
        Ok(())
    }
    pub fn with_dontcompress(&mut self) -> Result<(),String> {
        trace!("with_dontcompress");
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:388
            error!("dontembedname provided for non embed operation");
            return Err("the argument 'dontembedname' can only be used with the 'embed' command mode".to_string());
        }
        if self.compression_level.is_none(){
            debug!("Set compression level to 0 from dontcompress cli argument");
            self.compression_level = Some(0u8);
        } else {
            error!("compression has been already set");
            return Err("the arguments 'compress' and 'dontcompress' are conflictive".to_string());
        }
        Ok(())
    }
    pub fn with_dontembedname(&mut self) -> Result<(),String> {
        trace!("with_dontembedname");
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:388
            error!("dontcompress provided for non embed operation");
            return Err("the argument 'dontcompress' can only be used with the 'embed' command mode".to_string());
        }
        self.embed_name = false;
        Ok(())
    }
    pub fn with_radius(&mut self, radius: String) -> Result<(),String> {
        trace!("with_radius: {}", radius);
        self.radius = radius.parse::<u64>().unwrap();
        Ok(())
    }
    pub fn with_goal(&mut self, goal: String) -> Result<(),String> {
        trace!("with_goal: {}", goal);
        self.goal = goal.parse::<f64>().unwrap();
        Ok(())
    }
    pub fn with_marker(&mut self, marker: String) -> Result<(),String> {
        trace!("with_marker: {}", marker);
        self.marker = marker;
        Ok(())
    }
    pub fn with_force(&mut self) -> Result<(),String> {
        trace!("with_force");
        self.force = true;
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
        trace!("with_debug_printgraph");
        self.set_debug(super::DebugMode::PrintGraph);
        Ok(())
    }
    pub fn with_debug_printgmlgraph(&mut self, printgmlgraph: String) -> Result<(),String> {
        trace!("with_debug_printgmlgraph: {}", printgmlgraph);
        self.set_debug(super::DebugMode::PrintGmlGraph);
        Ok(())
    }
    pub fn with_debug_printgmlvertex(&mut self, printgmlvertex: String) -> Result<(),String> {
        trace!("with_debug_printgmlvertexstartvertex: {}", printgmlvertex);
        // TODO: Split printgmlvertex by ","
        // First value is RecDepth
        // Second value is StartVertex
        let val1 = 0u64;
        let val2 = 0u64;
        self.debug_mode = Some(super::DebugMode::PrintGmlVertex(val1,val2));
        Ok(())
    }
    pub fn with_debuglevel(&mut self, debuglevel: String) -> Result<(),String> {
        trace!("with_debuglevel: {}", debuglevel);
        self.set_debug(super::DebugMode::DebugLevel(debuglevel.parse::<u64>().unwrap()));
        Ok(())
    }
    pub fn with_debug_printstats(&mut self) -> Result<(),String> {
        trace!("with_debug_printstatsq");
        self.set_debug(super::DebugMode::PrintStats);
        Ok(())
    }
    pub fn with_check(&mut self) -> Result<(),String> {
        trace!("with_check");
        self.set_debug(super::DebugMode::Check);
        Ok(())
    }
    pub fn with_encryption(&mut self, encryption: String) -> Result<(),String> {
        trace!("with_encryption: {}", encryption);
        if self.command_mode != Some(super::CommandMode::Embed) { // Arguments.cc:193
            error!("encryption provided for non embed operation");
            return Err("the argument 'encryption' can only be used with the 'embed' command mode".to_string());
        }
        return Err("--encryption is not yet supported".to_string());
        Ok(())
    }
    // Validation related functions:
    /// `policy_set_stegofile_from_cover` sets stegofile to be the cover file
    /// when stegofile is not set and coverfile is set.
    fn policy_set_stegofile_from_cover(&mut self){
        self.stegofile = self.coverfile.clone();
        self.force = true;
    }
    /// `validate_build` checks the arguments passed are valid before returning the StegHide request.
    fn validate_build(&mut self) -> Result<(),String> {
        debug!("Validating build");
        if self.command_mode == Some(super::CommandMode::Embed) { // Arguments.cc:97
            if self.coverfile == Some("-".to_string()) && self.embedfile == Some("-".to_string()) {
                return Err("standard input cannot be used for cover data AND data to be embedded".to_string());
            }
            if self.stegofile.is_none() && self.coverfile.is_some() {
                self.policy_set_stegofile_from_cover();
            }
        }
        if self.command_mode == Some(super::CommandMode::Embed) || self.command_mode == Some(super::CommandMode::Extract) { // Arguments.cc:107
            if self.passphrase.is_none(){
                let mut double_check_passphrase = false;
                // prompt for passphrase
                if self.command_mode == Some(super::CommandMode::Embed){
                    if self.coverfile.is_none() || self.embedfile.is_none() {
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
            self.compression_level = Some(DEFAULT_COMPRESSION);
        }
        Ok(())
    }
    /// `build` finishes the builder for StepHideSetup after validating parameters
    pub fn build(mut self) -> super::StegHideSetup {
        match self.validate_build() {
            Ok(()) => trace!("Build params validated"),
            Err(err) => self.cli_exit_clap_invalidvalue(err),
        };
        super::StegHideSetup{
            passphrase: self.passphrase.unwrap(),
            compression_level: self.compression_level.unwrap(),
            command: self.command_mode,
            debug: self.debug_mode,
            // XXX: continue
        }
    }
}
pub fn parse_optional_args(builder: &mut StegHideCommandBuilder, matches: &clap::ArgMatches) -> Result<(),String> {
    let passphrase = matches.value_of("passphrase").unwrap();
    builder.with_passphrase(passphrase.to_string());
    if matches.is_present("embedfile"){
        let embedfile = matches.value_of("embedfile").unwrap();
        builder.with_embedfile(embedfile.to_string())?;
    }
    if matches.is_present("extractfile"){
        let extractfile = matches.value_of("extractfile").unwrap();
        builder.with_extractfile(extractfile.to_string());
    }
    if matches.is_present("coverfile"){
        let coverfile = matches.value_of("coverfile").unwrap();
        builder.with_coverfile(coverfile.to_string());
    }
    if matches.is_present("stegofile"){
        let stegofile = matches.value_of("stegofile").unwrap();
        builder.with_stegofile(stegofile.to_string());
    }
    if matches.is_present("infofile"){
        let infofile = matches.value_of("infofile").unwrap();
        builder.with_infofile(infofile.to_string());
    }
    if matches.is_present("nochecksum"){
        builder.with_nochecksum();
    }
    if matches.is_present("compress"){
        let compress = matches.value_of("compress").unwrap();
        builder.with_compress(compress.to_string());
    }
    if matches.is_present("dontcompress"){
        builder.with_dontcompress();
    }
    if matches.is_present("dontembedname"){
        builder.with_dontembedname();
    }
    if matches.is_present("radius"){
        let radius = matches.value_of("radius").unwrap();
        builder.with_radius(radius.to_string());
    }
    if matches.is_present("goal"){
        let goal = matches.value_of("goal").unwrap();
        builder.with_goal(goal.to_string());
    }
    if matches.is_present("marker"){
        let marker = matches.value_of("marker").unwrap();
        builder.with_marker(marker.to_string());
    }
    if matches.is_present("force"){
        builder.with_force();
    }
    if matches.is_present("printgraph"){
        builder.with_debug_printgraph();
    }
    if matches.is_present("printgmlgraph"){
        let printgmlgraph = matches.value_of("printgmlgraph").unwrap();
        builder.with_debug_printgmlgraph(printgmlgraph.to_string());
    }
    if matches.is_present("printgmlvertex"){
        let printgmlvertex = matches.value_of("printgmlvertex").unwrap();
        builder.with_debug_printgmlvertex(printgmlvertex.to_string());
    }
    if matches.is_present("debuglevel"){
        let debuglevel = matches.value_of("debuglevel").unwrap();
        builder.with_debuglevel(debuglevel.to_string());
    }
    if matches.is_present("printstats"){
        builder.with_debug_printstats();
    }
    if matches.is_present("check"){
        builder.with_check();
    }
    if matches.is_present("encryption"){
        let encryption = matches.value_of("encryption").unwrap();
        builder.with_encryption(encryption.to_string());
    }
    Ok(())
}
/// `parse_arguments` parses command line flags, sets up logging and timestamp
///  format
pub fn parse_arguments() -> Result<super::StegHideSetup,String> {
    // Parse command line arguments from cli.yml
    let cli_yaml = load_yaml!("cli.yml");
    let clap_args = App::from_yaml(cli_yaml).get_matches();
    let verbosity = clap_args.value_of("verbosity").map(|v| {
        v.parse::<usize>().unwrap_or_else(|_| {
            clap::Error {
                message: "invalid value for 'verbosity', should be a value from 0 to 4".to_string(),
                kind: clap::ErrorKind::InvalidValue,
                info: None,
            }.exit()
        })
    }).unwrap_or(1usize);
    let quiet = clap_args.is_present("quiet");
    let ts = clap_args.value_of("timestamp").map(|v| {
        stderrlog::Timestamp::from_str(v).unwrap_or_else(|_| {
            clap::Error {
                message: "invalid value for 'timestamp'".to_string(),
                kind: clap::ErrorKind::InvalidValue,
                info: None,
            }.exit()
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
    // info!("CRAP ARGS: {}",clap);
    let command_mode = clap_args.value_of("MODE").unwrap();
    let mut builder = StegHideCommandBuilder::new()
        .with_command(command_mode.to_string());
    for i in cli_yaml.iter() {
        println!("i: {}", i);
    }
    match parse_optional_args(&mut builder, &clap_args, cli_yaml) {
        Err(error_string) => {
            error!("Invalid values. Exiting.");
            clap::Error {
                message: error_string,
                kind: clap::ErrorKind::InvalidValue,
                info: None,
            }.exit()
        },
        Ok(()) => ()
    };
    Ok(builder.build())
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
    use super::*;
    #[test]
    fn it_validates_invalid_stdin_both_cover_embed() {
        let mut builder = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        builder.with_passphrase("aoeu".to_string());
        builder.with_coverfile("-".to_string());
        assert_eq!(builder.with_embedfile("-".to_string()), Ok(()));
        assert_eq!(builder.validate_build(), Err("standard input cannot be used for cover data AND data to be embedded".to_string()));
    }
    #[test]
    fn it_sets_stegofile_from_cover_when_empty() {
        let mut builder = StegHideCommandBuilder::new()
            .with_command("embed".to_string());
        builder.with_passphrase("aoeu".to_string());
        builder.with_coverfile("a".to_string());
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
        embed_none.with_coverfile("a".to_string());
        assert_eq!(
            embed_none.validate_build(),
            Err("if standard input is used, the passphrase must be specified on the command line".to_string())
        );
    }
    #[test]
    fn it_checks_passphrase_argument_needed_on_extract() {
        let mut cover_none = StegHideCommandBuilder::new()
            .with_command("extract".to_string());
        cover_none.with_extractfile("a".to_string());
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
}
