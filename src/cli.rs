// Copyright (c) 2021 Ghaith Hachem and Mathias Rieder
use clap::{ArgGroup, Parser};
use encoding_rs::Encoding;
use std::{ffi::OsStr, path::Path};

use crate::{ConfigFormat, ErrorFormat, FormatOption};

// => Set the default output format here:
const DEFAULT_FORMAT: FormatOption = FormatOption::Static;
const DEFAULT_OUTPUT_NAME: &str = "out";

pub type ParameterError = clap::Error;

#[derive(Parser, Debug)]
#[clap(
    group = ArgGroup::new("format"),
    about = "IEC61131-3 Structured Text compiler powered by Rust & LLVM ",
    version,
)]
pub struct CompileParameters {
    #[clap(
        short,
        long,
        name = "output-file",
        help = "Write output to <output-file>"
    )]
    pub output: Option<String>,

    #[clap(
        long = "check",
        group = "format",
        help = "Only validate input program (do not generate any code)"
    )]
    pub check_only: bool,

    #[clap(
        long = "ir",
        group = "format",
        help = "Emit IR (LLVM Intermediate Representation) as output"
    )]
    pub output_ir: bool,

    #[clap(
        long = "shared",
        group = "format",
        help = "Emit a shared object as output"
    )]
    pub output_shared_obj: bool,

    #[clap(
        long = "pic",
        group = "format",
        help = "Emit PIC (Position Independent Code) as output"
    )]
    pub output_pic_obj: bool,

    #[clap(long = "static", group = "format", help = "Emit an object as output")]
    pub output_obj_code: bool,

    #[clap(
        long = "relocatable",
        group = "format",
        help = "Emit an object as output"
    )]
    pub output_reloc_code: bool,

    #[clap(
        long = "bc",
        group = "format",
        help = "Emit binary IR (binary representation of LLVM-IR) as output"
    )]
    pub output_bit_code: bool,

    #[clap(short = 'c', help = "Do not link after compiling object code")]
    pub skip_linking: bool,

    #[clap(
        long,
        name = "target-triple",
        help = "A target-tripple supported by LLVM"
    )]
    pub target: Option<String>,

    #[clap(
        long,
        name = "encoding",
        help = "The file encoding used to read the input-files, as defined by the Encoding Standard",
        parse(try_from_str = parse_encoding),
    )]
    pub encoding: Option<&'static Encoding>,

    #[clap(
        name = "input-files",
        help = "Read input from <input-files>, may be a glob expression like 'src/**/*' or a sequence of files",
        required = true,
        min_values = 1
    )]
    // having a vec allows bash to resolve *.st itself
    pub input: Vec<String>,

    #[clap(
        name = "library-path",
        long,
        short = 'L',
        help = "Search path for libraries, used for linking"
    )]
    pub library_pathes: Vec<String>,

    #[clap(name = "library", long, short = 'l', help = "Library name to link")]
    pub libraries: Vec<String>,

    #[clap(long, name = "sysroot", help = "Path to system root, used for linking")]
    pub sysroot: Option<String>,

    #[clap(
        name = "include",
        long,
        short = 'i',
        help = "Include source files for external functions"
    )]
    pub includes: Vec<String>,

    #[clap(
        name = "hardware-conf",
        long,
        help = "Generate Hardware configuration files to the given location. 
    Format is detected by extenstion.
    Supported formats : json, toml",
    parse(try_from_str = validate_config)
    ) ]
    pub hardware_config: Option<String>,

    #[clap(
        name = "optimization",
        long,
        short = 'O',
        help = "Optimization level",
        arg_enum,
        default_value = "default"
    )]
    pub optimization: crate::OptimizationLevel,

    #[clap(
        name = "error-format",
        long,
        help = "Set format for error reporting",
        arg_enum,
        default_value = "rich"
    )]
    pub error_format: ErrorFormat,
}

fn parse_encoding(encoding: &str) -> Result<&'static Encoding, String> {
    Encoding::for_label(encoding.as_bytes()).ok_or(format!("Unknown encoding {}", encoding))
}

fn validate_config(config_name: &str) -> Result<String, String> {
    if get_config_format(config_name).is_some() {
        Ok(config_name.to_string())
    } else {
        Err(format!(
            r#"Cannot identify format type for {}, valid extensions : "json", "toml""#,
            config_name
        ))
    }
}

pub fn get_config_format(name: &str) -> Option<ConfigFormat> {
    let ext = name.split('.').last();
    match ext {
        Some("json") => Some(ConfigFormat::JSON),
        Some("toml") => Some(ConfigFormat::TOML),
        _ => None,
    }
}

impl CompileParameters {
    pub fn parse(args: Vec<String>) -> Result<CompileParameters, ParameterError> {
        CompileParameters::try_parse_from(args)
    }

    // convert the scattered bools from structopt into an enum
    pub fn output_format(&self) -> Option<FormatOption> {
        if self.output_bit_code {
            Some(FormatOption::Bitcode)
        } else if self.output_ir {
            Some(FormatOption::IR)
        } else if self.output_pic_obj {
            Some(FormatOption::PIC)
        } else if self.output_shared_obj {
            Some(FormatOption::Shared)
        } else if self.output_obj_code {
            Some(FormatOption::Static)
        } else if self.output_reloc_code {
            Some(FormatOption::Relocatable)
        } else {
            None
        }
    }

    /// return the selected output format, or the default if none.
    pub fn output_format_or_default(&self) -> Option<FormatOption> {
        // structop makes sure only one or zero format flags are
        // selected. So if none are selected, the default is chosen

        if self.check_only {
            // if the --check flag is selected, we aren't supposted
            // to generate any code, so none of the format options
            // is valid.
            return None;
        }

        Some(self.output_format().unwrap_or(DEFAULT_FORMAT))
    }

    /// return the output filename with the correct ending
    pub fn output_name(&self) -> Option<String> {
        let out_format = self.output_format_or_default();

        if let Some(n) = &self.output {
            Some(n.to_string())
        } else {
            let ending = match out_format {
                Some(out_format) => match out_format {
                    FormatOption::Bitcode => ".bc",
                    FormatOption::Relocatable => ".o",
                    FormatOption::Static if self.skip_linking => ".o",
                    FormatOption::Static => "",
                    FormatOption::Shared | FormatOption::PIC => ".so",
                    FormatOption::IR => ".ir",
                },
                None => "",
            };

            let output_name = self.input.first().map(String::as_str);
            let basename = output_name
                .and_then(|it| Path::new(it).file_stem())
                .and_then(OsStr::to_str)
                .unwrap_or(DEFAULT_OUTPUT_NAME);
            Some(format!("{}{}", basename, ending))
        }
    }

    pub fn config_format(&self) -> Option<ConfigFormat> {
        self.hardware_config.as_deref().and_then(get_config_format)
    }
}

#[cfg(test)]
mod cli_tests {
    use super::CompileParameters;
    use crate::{ConfigFormat, ErrorFormat, FormatOption, OptimizationLevel};
    use clap::ErrorKind;
    use pretty_assertions::assert_eq;

    fn expect_argument_error(args: Vec<String>, expected_error_kind: ErrorKind) {
        let params = CompileParameters::parse(args.clone());
        match params {
            Err(e) => {
                assert_eq!(e.kind(), expected_error_kind);
            }
            Ok(p) => panic!(
                "expected error, but found none. arguments: {:?}. params: {:?}",
                args, p
            ),
        }
    }
    macro_rules! vec_of_strings {
        ($($x:expr),*) => (vec!["rustyc".to_string(), $($x.to_string()),*]);
    }

    #[test]
    fn missing_parameters_results_in_error() {
        // no arguments
        expect_argument_error(vec![], ErrorKind::MissingRequiredArgument);
        // no input file
        expect_argument_error(vec_of_strings!["--ir"], ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn multiple_output_formats_results_in_error() {
        expect_argument_error(
            vec_of_strings!["input.st", "--check", "--shared"],
            ErrorKind::ArgumentConflict,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "--shared"],
            ErrorKind::ArgumentConflict,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "--shared", "--pic"],
            ErrorKind::ArgumentConflict,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "--shared", "--pic", "--bc"],
            ErrorKind::ArgumentConflict,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "--relocatable"],
            ErrorKind::ArgumentConflict,
        );
    }

    #[test]
    fn unknown_arguments_results_in_error() {
        expect_argument_error(
            vec_of_strings!["input.st", "--unknown"],
            ErrorKind::UnknownArgument,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "--unknown"],
            ErrorKind::UnknownArgument,
        );
        expect_argument_error(
            vec_of_strings!["input.st", "--ir", "-u"],
            ErrorKind::UnknownArgument,
        );
    }

    #[test]
    fn valid_output_files() {
        //short -o
        let parameters =
            CompileParameters::parse(vec_of_strings!("input.st", "--ir", "-o", "myout.out"))
                .unwrap();
        assert_eq!(parameters.output_name().unwrap(), "myout.out".to_string());

        //long --output
        let parameters = CompileParameters::parse(vec_of_strings!(
            "input.st",
            "--ir",
            "--output",
            "myout2.out"
        ))
        .unwrap();
        assert_eq!(parameters.output_name().unwrap(), "myout2.out".to_string());
    }

    #[test]
    fn test_default_output_names() {
        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st", "--ir")).unwrap();
        assert_eq!(parameters.output_name().unwrap(), "alpha.ir".to_string());

        let parameters = CompileParameters::parse(vec_of_strings!("bravo", "--shared")).unwrap();
        assert_eq!(parameters.output_name().unwrap(), "bravo.so".to_string());

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/charlie.st", "--pic")).unwrap();
        assert_eq!(parameters.output_name().unwrap(), "charlie.so".to_string());

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/delta.st", "--static", "-c"))
                .unwrap();
        assert_eq!(parameters.output_name().unwrap(), "delta.o".to_string());

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/echo", "--bc")).unwrap();
        assert_eq!(parameters.output_name().unwrap(), "echo.bc".to_string());

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/echo.st")).unwrap();
        assert_eq!(parameters.output_name().unwrap(), "echo".to_string());
    }

    #[test]
    fn test_target_triple() {
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "--target", "x86_64-linux-gnu"))
                .unwrap();

        assert_eq!(parameters.target, Some("x86_64-linux-gnu".to_string()));
    }

    #[test]
    fn test_optimization_levels() {
        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st")).unwrap();

        assert_eq!(parameters.optimization, OptimizationLevel::Default);
        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st", "-Onone")).unwrap();

        assert_eq!(parameters.optimization, OptimizationLevel::None);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "--optimization", "none"))
                .unwrap();
        assert_eq!(parameters.optimization, OptimizationLevel::None);

        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st", "-Oless")).unwrap();

        assert_eq!(parameters.optimization, OptimizationLevel::Less);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "--optimization", "less"))
                .unwrap();
        assert_eq!(parameters.optimization, OptimizationLevel::Less);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "-Odefault")).unwrap();

        assert_eq!(parameters.optimization, OptimizationLevel::Default);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "--optimization", "default"))
                .unwrap();
        assert_eq!(parameters.optimization, OptimizationLevel::Default);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "-Oaggressive")).unwrap();

        assert_eq!(parameters.optimization, OptimizationLevel::Aggressive);
        let parameters =
            CompileParameters::parse(vec_of_strings!("alpha.st", "--optimization", "aggressive"))
                .unwrap();
        assert_eq!(parameters.optimization, OptimizationLevel::Aggressive);
    }

    #[test]
    fn test_default_format() {
        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st", "--check")).unwrap();
        assert_eq!(parameters.output_format_or_default(), None);

        let parameters = CompileParameters::parse(vec_of_strings!("alpha.st", "--ir")).unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(FormatOption::IR)
        );

        let parameters = CompileParameters::parse(vec_of_strings!("bravo", "--shared")).unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(FormatOption::Shared)
        );

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/charlie.st", "--pic")).unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(FormatOption::PIC)
        );

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/delta.st", "--static"))
                .unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(FormatOption::Static)
        );

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/echo", "--bc")).unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(FormatOption::Bitcode)
        );

        let parameters =
            CompileParameters::parse(vec_of_strings!("examples/test/echo.st")).unwrap();
        assert_eq!(
            parameters.output_format_or_default(),
            Some(super::DEFAULT_FORMAT)
        );
    }

    #[test]
    fn encoding_resolution() {
        let parameters =
            CompileParameters::parse(vec_of_strings!("input.st", "--ir", "--encoding", "cp1252"))
                .unwrap();
        assert_eq!(parameters.encoding, Some(encoding_rs::WINDOWS_1252));
        let parameters = CompileParameters::parse(vec_of_strings!(
            "input.st",
            "--ir",
            "--encoding",
            "windows-1252"
        ))
        .unwrap();
        assert_eq!(parameters.encoding, Some(encoding_rs::WINDOWS_1252));
    }

    #[test]
    fn invalid_encoding_resolution() {
        expect_argument_error(
            vec_of_strings!("input.st", "--ir", "--encoding", "invalid"),
            ErrorKind::ValueValidation,
        );
    }

    #[test]
    fn valid_output_formats() {
        let parameters = CompileParameters::parse(vec_of_strings!("input.st", "--ir")).unwrap();
        assert!(parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);

        let parameters = CompileParameters::parse(vec_of_strings!("input.st", "--bc")).unwrap();
        assert!(!parameters.output_ir);
        assert!(parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);

        let parameters = CompileParameters::parse(vec_of_strings!("input.st", "--static")).unwrap();
        assert!(!parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);

        let parameters = CompileParameters::parse(vec_of_strings!("input.st", "--pic")).unwrap();
        assert!(!parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);

        let parameters = CompileParameters::parse(vec_of_strings!("input.st", "--shared")).unwrap();
        assert!(!parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);

        let parameters =
            CompileParameters::parse(vec_of_strings!("input.st", "--relocatable")).unwrap();
        assert!(!parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(parameters.output_reloc_code);

        let parameters = CompileParameters::parse(vec_of_strings!("input.st")).unwrap();
        assert!(!parameters.output_ir);
        assert!(!parameters.output_bit_code);
        assert!(!parameters.output_obj_code);
        assert!(!parameters.output_pic_obj);
        assert!(!parameters.output_shared_obj);
        assert!(!parameters.output_reloc_code);
    }

    #[test]
    fn library_path_added() {
        let parameters = CompileParameters::parse(vec_of_strings!(
            "input.st",
            "--library-path",
            "xxx",
            "-L",
            "test",
            "-L.",
            "-L/tmp"
        ))
        .unwrap();
        assert_eq!(parameters.library_pathes, vec!["xxx", "test", ".", "/tmp"]);
    }

    #[test]
    fn libraries_added() {
        let parameters = CompileParameters::parse(vec_of_strings!(
            "input.st",
            "-l",
            "test",
            "-lc",
            "--library",
            "xx"
        ))
        .unwrap();
        assert_eq!(parameters.libraries, vec!["test", "c", "xx"]);
    }

    #[test]
    fn cli_supports_version() {
        match CompileParameters::parse(vec_of_strings!("input.st", "--version")) {
            Ok(_) => panic!("expected version output, but found OK"),
            Err(e) => assert_eq!(e.kind(), ErrorKind::DisplayVersion),
        }
    }

    #[test]
    fn cli_supports_help() {
        match CompileParameters::parse(vec_of_strings!("input.st", "--help")) {
            Ok(_) => panic!("expected help output, but found OK"),
            Err(e) => assert_eq!(e.kind(), ErrorKind::DisplayHelp),
        }
    }

    #[test]
    fn sysroot_added() {
        let parameters =
            CompileParameters::parse(vec_of_strings!("input.st", "--sysroot", "path/to/sysroot"))
                .unwrap();
        assert_eq!(parameters.sysroot, Some("path/to/sysroot".to_string()));
    }

    #[test]
    fn include_files_added() {
        let parameters = CompileParameters::parse(vec_of_strings!(
            "input.st",
            "-i",
            "include1",
            "-i",
            "include2",
            "--include",
            "include3"
        ))
        .unwrap();
        assert_eq!(
            parameters.includes,
            vec!["include1", "include2", "include3"]
        );
    }

    #[test]
    fn config_option_set() {
        let parameters =
            CompileParameters::parse(vec_of_strings!("foo", "--hardware-conf=conf.json")).unwrap();
        assert_eq!(parameters.hardware_config, Some("conf.json".to_string()));
        assert_eq!(parameters.config_format().unwrap(), ConfigFormat::JSON);
        let parameters =
            CompileParameters::parse(vec_of_strings!("foo", "--hardware-conf=conf.toml")).unwrap();
        assert_eq!(parameters.hardware_config, Some("conf.toml".to_string()));
        assert_eq!(parameters.config_format().unwrap(), ConfigFormat::TOML);

        expect_argument_error(
            vec_of_strings!("foo", "--hardware-conf=foo"),
            ErrorKind::ValueValidation,
        );
        expect_argument_error(
            vec_of_strings!("foo", "--hardware-conf=conf.foo"),
            ErrorKind::ValueValidation,
        );
        expect_argument_error(
            vec_of_strings!("foo", "--hardware-conf=conf.xml"),
            ErrorKind::ValueValidation,
        );
    }

    #[test]
    fn error_format_default_set() {
        // make sure the default error format is set
        let params = CompileParameters::parse(vec_of_strings!("input.st")).unwrap();
        assert_eq!(params.error_format, ErrorFormat::Rich);
    }

    #[test]
    fn error_format_set() {
        // set clang as error format
        let params =
            CompileParameters::parse(vec_of_strings!("input.st", "--error-format=clang")).unwrap();
        assert_eq!(params.error_format, ErrorFormat::Clang);
        // set invalid error format
        expect_argument_error(
            vec_of_strings!("input.st", "--error-format=none"),
            ErrorKind::InvalidValue,
        );
    }
}
