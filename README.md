# wait-nscd

This is a small tool that waits for nscd to start up and return the correct data.
It interfaces with the nscd socket directly and doesn't require any C libraries.

The check is performed by asking for a user via nscd and comparing the results with expected values.
If an error is returned or the data is not as expected, the tool sleeps and tries again.

Errors are printed and if everything goes right, nothing is printed to screen

## Usage

```
wait-nscd
Wait for nscd to return the correct data

USAGE:
    wait-nscd [OPTIONS]

OPTIONS:
    -h, --help                           Print help information
    -i, --expected-uid <EXPECTED_UID>    UID to expect from the lookup [default: 0]
    -m, --sleep-millis <SLEEP_MILLIS>    Milliseconds to sleep between tries [default: 100]
    -s, --nscd-socket <NSCD_SOCKET>      nscd socket to connect to [default: /var/run/nscd/socket]
    -u, --username <USERNAME>            Username to look up via nscd [default: root]
```
