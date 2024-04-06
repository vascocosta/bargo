# Bargo

BASIC build system and package manager.

## Features

* Automatic line numbering
* Customisable line numbering
* Customisable newline chars
* Dependency/module management
* Project creation
* Project build

## Build

To build `bargo` you need the `Rust toolchain` as well as these `dependencies`:

* serde = "1.0.197"
* toml = "0.8.12"

Follow these steps to fetch and compile the source of `bargo` and its `dependencies`:

```
git clone https://github.com/vascocosta/bargo.git

cd bargo

cargo build --release
```

## Install

Simply copy `bargo/target/release/bargo` to a folder in your path (ex: `$HOME/bin`).

## Usage example

Bargo allows you to create a project template for your BASIC program, so that editing your source code is simpler.

Follow the steps below to create a new project template, cd into the project, edit some files with your favourite editor (replace $editor with your favourite editor) and finally build it.

```
bargo new age
cd age
$editor Bargo.toml 
$editor src/main.bas
$editor src/utils.bas
bargo build
```

1. Create a new project called `age`.
2. Change the current folder to your project folder called `age`.
3. Edit `Bargo.toml`, which is the configuration file of your project.
4. Edit `src/main.bas` which by convention is where you define your main program.
5. Edit `src/utils.bas` which is a dependency/module used by your main program.
6. Build your project.

The final step builds your project by merging `src/main.bas` with all the dependencies/modules your project uses into a single file called `age.bas` at the root of your project folder and automatically numbering the lines for you.

In this simple example there's only `src/utils.bas`, but you could use any number of dependencies/modules. For each dependency/module you should create a bas file with the name of that dependency/module inside `src` and add the name to the `[dependencies]` section of `Bargo.toml`, plus its vesion. For now the version isn't important, but it will be used in the future. For example, if you were to add another dependency/module called `math`, you would need to create `src/math.bas` with math related procedures/functions and then add `math = "0.1.0"` to `Bargo.toml` under the `[dependencies]` section.

Below you can see listing for all the files used in this example, plus the final source code of `age.bas` that is generated for you.

### Bargo.toml

Your BASIC project is configured by editing `age/Bargo.toml`:

```
[package]
name = "age"
carriage_return = true
numbering = 10
version = "0.1.0"

[dependencies]
utils = "0.1.0
```

### src/main.bas

Your main source file is created by editing `age/src/main.bas`:

```
PROC_INIT_SCREEN
PRINT "Hello!"
PRINT
PRINT "What is the current year?"
INPUT CURRENT_YEAR%
PRINT "What year were you born in?"
INPUT BIRTH_YEAR%
AGE% = FN_CALCULATE_AGE%(CURRENT_YEAR%, BIRTH_YEAR%)
PRINT "You are " + STR$(AGE%) + " years old."
END
```

### src/utils.bas

```
DEF PROC_INIT_SCREEN
MODE 3
CLS
ENDPROC
:
DEF FN_CALCULATE_AGE%(CURRENT_YEAR%, BIRTH_YEAR%)
= CURRENT_YEAR% - BIRTH_YEAR%
ENDDEF
```

### age.bas

```basic
 10 PROC_INIT_SCREEN
 20 PRINT "Hello!"
 30 PRINT
 40 PRINT "What is the current year?"
 50 INPUT CURRENT_YEAR%
 60 PRINT "What year were you born in?"
 70 INPUT BIRTH_YEAR%
 80 AGE% = FN_CALCULATE_AGE%(CURRENT_YEAR%, BIRTH_YEAR%)
 90 PRINT "You are " + STR$(AGE%) + " years old."
100 END
110 :
120 REM ============================================================================
130 REM IMPORT UTILS.BAS
140 REM ============================================================================
150 :
160 DEF PROC_INIT_SCREEN
170 MODE 3
180 CLS
190 ENDPROC
200 :
210 DEF FN_CALCULATE_AGE%(CURRENT_YEAR%, BIRTH_YEAR%)
220 = CURRENT_YEAR% - BIRTH_YEAR%
230 ENDDEF
```