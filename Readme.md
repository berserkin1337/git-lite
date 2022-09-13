# Building
```bash
cargo build 
```
# Running
```bash
./target/debug/git-lite
git_lite 
implementation of git in rust.

USAGE:
    git-lite [SUBCOMMAND]

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add            Add file contents to the index
    cat-file       Provide content or type and size information for repository objects
    commit         Record changes to the repository
    hash-object    Compute object ID and optionally creates a blob from a file
    help           Print this message or the help of the given subcommand(s)
    init           Creates a new git repository or reinitializes an existing one.
    ls-files       Lists the files in the git index
```
# Initiallizing a repository
```bash
$ git_lite init
```

# Adding a file to the index
```bash
$ echo "hello world" > hello.txt
$ git_lite add hello.txt
```
# View the files in the index
```bash
$ git_lite ls-files
hello.txt
```
# Commiting the files in the index
```bash
$ git_lite commit -m "initial commit" -a "aviral"
Commited to master: 79873421ae6fb1a30c4faeb5b5fe54ad8f8e89eb
```

# References
- [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [the-git-parable](https://tom.preston-werner.com/2009/05/19/the-git-parable.html)
- [git from the bottom up](https://jwiegley.github.io/git-from-the-bottom-up/)
- [git from the inside out](https://codewords.recurse.com/issues/two/git-from-the-inside-out)
- [Write yourself a Git!](https://wyag.thb.lt/)