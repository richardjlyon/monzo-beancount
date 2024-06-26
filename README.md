# Monzo Beancount

A rust application to convert Monzo transactions to Beancount format.

## Installation

```bash
cargo install --git https://github.com/richardjlyon/monzo-beancount
```

## Documentation

[](https://richardjlyon.github.io/monzo-beancount/)
[crates]()

## Usage

```shell
> monzo-beancount init # initialises the file system in the home directory
> cd ~/beancount
> monzo-beancount generate # (re)generates the main Beancount file
> bean-check main.beancount # checks the file for errors
> bean-web main.beancount # starts the web server
> (open URL: http://localhost:8080)
```

## Configuration

### Authorisation

[TBA]

### Configuration File

`beancount.yaml` in the beancount folder root allows you to configure the app to your
accounts. See [documentation](https://richardjlyon.github.io/monzo-beancount/configuration/) for details.

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)

### Change Log
