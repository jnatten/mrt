# mrt - Multi Repo Tool

A tool to interact with multiple repositories.

(By executing the specified commands in each context)

This is not a finished product and there is a chance you will encounter bugs or problems. 

Reports of [issues](https://github.com/jnatten/mrt/issues/new) are really appreciated, along with pull-requests and suggestions for new features and how to improve the code (Rust is still very new to me).

### Installation

Right now there aren't any releases, but you can compile the code yourself with these few steps:

1. Install rust (I like [rustup](https://www.rust-lang.org/tools/install) for this)
2. Clone this repository:

    `$ git clone https://github.com/jnatten/mrt.git`
3. Compile the source code with cargo from within the repository:

    `$ cd mrt && cargo build --release`
4. Move `mrt` executable to somewhere on PATH, On linux this can be something like /usr/bin/mrt

    `$ mv target/release/mrt /usr/bin/mrt`
    
5. Done! See [usage](#usage):

### Usage
```
$ mrt -h
Multi Repo Tool 0.0.1

USAGE:
    mrt [FLAGS] [OPTIONS] [+tag ..]

FLAGS:
    -c, --continuous-output    Will make output from commands executed in parallel with --parallel argument print to
                               terminal before every command has been executed.
    -h, --help                 Prints help information
    -l, --list-tags            List all specified +tag's and paths that are tagged...
    -p, --parallel             Execute at each tagged path in parallel
                               This stores output until all executions are finished and then prints them in sequence,
                               unless --continuous-output specified.
    -V, --version              Prints version information

OPTIONS:
    -a, --add-tag <TAG_NAME>...    Adds the current directory with specified +tag
    -d, --del-tag <TAG_NAME>...    Deletes the current directory with specified +tag

EXAMPLES:
    # Tag current directory with tag `backend`
    $ mrt -a backend

    # Remove tag `backend` from current directory
    $ mrt -d backend

    # List tagged directories
    $ mrt -l

    # Execute command in all directories tagged with `backend`
    $ mrt +backend sed -i 's/someversion = "1.0.0"/someversion = "1.2.0"/g build.sbt

    # Execute command in all directories tagged with `backend` in parallel
    $ mrt -p +backend git pull

    # Execute command in all directories tagged with `backend` and `frontend` in parallel
    $ mrt -p +backend +frontend git pull
 
```

### Why?

I work on many repositories with similar code in some sort of a microservice environment. 
I felt the need to reduce the overhead of maintaining several repositories with similarities.

Also I think rust is an interesting language which I want to learn more about.

### Development

- **Compile**: `cargo build` 
- **Compile + Run**: `cargo run -- -h` 
- **Run tests**: `cargo test`

### Alternatives

This is not a new idea and similar projects exist.
This list is probably not complete, but here are a few alternatives and a short summary of why I still felt compelled to make this.

- https://github.com/mixu/gr - Not actively maintained, no parallelization.
- https://github.com/tobru/myrepos - Not actively maintained, seems to be only for version control.
