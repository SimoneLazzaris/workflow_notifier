# Workflow notifier

A webhook handler that receives POSTs from github webhooks. If:
- it is about a workflow
- the branch is `master` or `main`
- the conclusion is not a `success` 
it will thenb call a mattermost webhook to notify the failure.

The notification will have the repository name and branch, the job url and the status.

## Configuration

There are a few command line options to configure the listener:

```
Usage: workflow_notifier [OPTIONS]

Options:
  -a, --address <ADDRESS>  [default: 127.0.0.1]
  -p, --port <PORT>        [default: 8080]
  -w, --webhook <WEBHOOK>  [default: http://mattermost/hook]
  -h, --help               Print help
```

The only one that *really* has to be setup is the mattermost webhook.

### Logging
This project uses (indirectly) env_logger, so to enable logging you can set the `RUST_LOG` environmental variable to the desired level, i.e. `RUST_LOG=debug` or `RUST_LOG=workflow_notifier=debug`.

