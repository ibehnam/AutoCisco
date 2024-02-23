# AutoCisco (My First Project in Rust)

CMU’s VPN requires SSO login, meaning that you need to enter your Andrew ID and password every time you want to use the VPN. This is a tedious task and I decided to automate the process. The result is **AutoCisco**: a simple CLI tool (i.e., you must run it in terminal) that automatically opens the “Cisco AnyConnect Secure Mobility Client” app, presses “Connect”, and then enters your username and password in the SSO login page.

- Note that AutoCisco runs on macOS only. If you’re using Windows or Linux (*by choice*), you’re probably not the type of person who likes convenience anyway 😬🙈

# Installation

Simply use Homebrew:

```bash
brew tap ibehnam/homebrew-packages
brew install autocisco
```

# Usage



https://github.com/ibehnam/AutoCisco/assets/58621210/2348b66f-976c-48a1-85bc-b78365556ad0


## The First Time

The first time you run AutoCisco, you should enter your username and password like so:

```bash
./AutoCisco --username <your-andrew-ID> --password <your-password>
```

You can also see more help by using `./AutoCisco --help`. AutoCisco saves these credentials in `$HOME/.vpn_credentials`.

- The program is not case-sensitive, so `autocisco` works too.
- You can also assign an alias to the command, like:

```bash
alias ac="/path/to/AutoCisco --username <your-andrew-ID> --password <your-password>"
```

This means you can now just run `ac` in terminal!

## Next Times

You don’t need to type your username and password in terminal anymore. Just do:

```bash
./AutoCisco
```

## Updating Your Username/Password

Just run AutoCisco with your new username and password (like the first time) with `--username` and `--password` arguments. AutoCisco will update the `$HOME/.vpn_credentials` file automatically.

## Bonus

If you give permissions to your terminal, you can even double-click AutoCisco and run it like a normal app.
