extern crate argparse;
extern crate subprocess;

#[macro_use]
extern crate garden;


#[derive(Debug)]
enum Command {
    Add,
    Cmd,
    Exec,
    Help,
    Init,
    List,
    Shell,
    Status,
}

impl std::default::Default for Command {
    fn default() -> Self { Command::Help }
}

impl_display_brief!(Command);


#[derive(Default)]
struct CommandOptions {
    args: Vec<String>,
    debug: std::collections::HashSet<String>,
    debug_str: String,
    filename: Option<std::path::PathBuf>,
    filename_str: String,
    keep_going: bool,
    quiet: bool,
    subcommand: Command,
    verbose: bool,
}

impl CommandOptions {
    fn update(&mut self) {
        if self.filename_str.len() > 0 {
            self.filename = Some(std::path::PathBuf::from(&self.filename_str));
        }

        for debug_name in self.debug_str.split(",") {
            self.debug.insert(debug_name.to_string());
        }
    }

    fn is_debug(&self, name: &str) -> bool {
        return self.debug.contains(name);
    }
}


impl std::str::FromStr for Command {
    type Err = ();  // For the FromStr trait

    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "add" => Ok(Command::Add),
            "do" => Ok(Command::Cmd),
            "cmd" => Ok(Command::Cmd),
            "ex" => Ok(Command::Exec),
            "exec" => Ok(Command::Exec),
            "help" => Ok(Command::Help),
            "init" => Ok(Command::Init),
            "list" => Ok(Command::List),
            "ls" => Ok(Command::List),
            "run" => Ok(Command::Cmd),
            "sh" => Ok(Command::Shell),
            "shell" => Ok(Command::Shell),
            "st" => Ok(Command::Status),
            "stat" => Ok(Command::Status),
            "status" => Ok(Command::Status),
            _ => Err(()),
        }
    }
}

fn garden_help(options: &mut CommandOptions) {
    let cmd_path = match std::env::current_exe() {
        Err(_) => std::path::PathBuf::from("garden"),
        Ok(path) => path,
    };
    let mut help_cmd = vec!(cmd_path);

    let mut command = String::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden help - command documentation");
        ap.stop_on_first_argument(true);

        ap.refer(&mut command)
            .add_argument("command", argparse::Store,
                          "Command help to display");

        options.args.insert(0, "garden help".to_string());
        ap.parse(options.args.to_vec(),
                 &mut std::io::stdout(), &mut std::io::stderr())
            .map_err(|c| std::process::exit(c)).ok();
    }

    // garden help foo -> garden foo --help
    if !command.is_empty() {
        help_cmd.push(std::path::PathBuf::from(command));
    }

    help_cmd.push(std::path::PathBuf::from("--help"));

    if options.verbose {
        debug!("help command");
        let mut i: i32 = 0;
        for arg in &help_cmd {
            debug!("help_cmd[{:02}] = {:?}", i, arg);
            i += 1;
        }
    }

    std::process::exit(garden::cmd::run(&help_cmd));
}


fn garden_cmd(options: &mut CommandOptions) {
    options.args.insert(0, "garden run".to_string());

    let mut expr = String::new();
    let mut commands: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden cmd - run preset commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to exec (tree expression)");

        ap.refer(&mut commands).required()
            .add_argument("commands", argparse::List,
                          "commands to run over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut config = garden::config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", config);
    }
    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("expr: {}", expr);
        debug!("commands: {:?}", commands);
    }

    garden::cmds::cmd::main(
        &mut config, options.quiet, options.verbose,
        options.keep_going, expr, &commands);
}


fn garden_exec(options: &mut CommandOptions) {
    options.args.insert(0, "garden exec".to_string());

    let mut expr = String::new();
    let mut command: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden exec - run commands inside gardens");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to exec (tree expression)");

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List,
                          "command to run over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut config = garden::config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", config);
    }
    if options.is_debug("exec") {
        debug!("subcommand: exec");
        debug!("expr: {}", expr);
        debug!("command: {:?}", command);
    }

    garden::cmds::exec::main(
        &mut config, options.quiet, options.verbose, expr, &command);
}

fn garden_list(options: &mut CommandOptions) {
    let config = garden::config::new(&options.filename, options.verbose);

    if !config.gardens.is_empty() {
        println!("gardens:");
        print!("    ");
        for garden in &config.gardens {
            print!("{} ", garden.name);
        }
        println!("");
    }

    if !config.groups.is_empty() {
        println!("groups:");
        print!("    ");
        for group in &config.groups {
            print!("{} ", group.name);
        }
        println!("");
    }

    if !config.trees.is_empty() {
        println!("trees:");
        print!("    ");
        for tree in &config.trees{
            print!("{} ", tree.name);
        }
        println!("");
    }
}


fn main() {
    let mut options = CommandOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - git tree organizer");

        ap.refer(&mut options.filename_str)
            .add_option(&["-c", "--config"], argparse::Store,
                        "specify the config file to use");

        ap.refer(&mut options.debug_str)
            .add_option(&["-d", "--debug"], argparse::Store,
                        "debug categories to enable (comma-separated)");

        ap.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue, "be verbose");

        ap.refer(&mut options.quiet)
            .add_option(&["-q", "--quiet"],
                        argparse::StoreTrue, "be quiet");

        ap.refer(&mut options.subcommand).required()
            .add_argument("command", argparse::Store,
                "command to run {cmd, exec, help, ls, init, shell, status}");

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List,
                "sub-command arguments");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    options.update();

    match options.subcommand {
        Command::Add => garden_help(&mut options),
        Command::Help => garden_help(&mut options),
        Command::Cmd => garden_cmd(&mut options),
        Command::Exec => garden_exec(&mut options),
        Command::Init => garden_help(&mut options),
        Command::List => garden_list(&mut options),
        Command::Status => garden_help(&mut options),
        Command::Shell => garden_help(&mut options),
    }
}
