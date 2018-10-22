# replacer
Replace text in files using regex pattern.

    $ replacer --help
    replacer 0.1.0
    Oleg Khryptul <okreptul@yahoo.com>
    Replace text in the files using regex pattern.
    Search in the specified file or in all files of the folder recursively.
    Supports multiline pattern replacement.

    USAGE:
        replacer <pattern> <replacement> [input]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -r, --replace <replacement>    Replacement string (rust regex). Do only pattern matching if not specified.

    ARGS:
        <pattern>        Pattern string (rust regex)
        <input>          Input file or starting directory. Searches in the current directory if not specified.
