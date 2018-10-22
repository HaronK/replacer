# replacer
Replace text in files using regex pattern.

    $ replacer --help
    replacer 0.1.1
    Oleg Khryptul <okreptul@yahoo.com>
    Replace text in the files using regex pattern.
    Search in the specified file or in all files of the folder recursively.
    Supports multiline pattern replacement.

    USAGE:
        replacer [FLAGS] [OPTIONS] <text_pattern> [inputs]...

    FLAGS:
        -h, --help         Prints help information
        -d, --show-diff    Show replaced or matched lines.
        -V, --version      Prints version information

    OPTIONS:
        -f, --file <file_pattern>      Pattern string for the file name (rust regex).
        -r, --replace <replacement>    Replacement string (rust regex). Do only pattern matching if not specified.

    ARGS:
        <text_pattern>    Pattern string for the text (rust regex).
        <inputs>...       Input files and/or starting directories. Searches in the current directory if not specified.