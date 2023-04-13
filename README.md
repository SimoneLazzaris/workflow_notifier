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

## Github configuration

To use this hook, first you have to make the endpoint publicly reachable from github. You can use [ngrok](https://ngrok.com/) for that matter, or simply have a public DNS name (and TLS certificate) that resolve to an handler instance.

Then, to activate the webhook for a repository in github, you need to go to the repository `Settings` -> `Webhooks` -> `Add webhook`. There, you enter the webhook url in the `Payload URL` textbox, select `application/json` as `Content Type`,  choose `Let me select individual events` and enable the `Workflow runs` events.

## kubernetes deployment

A reference resource definition `.yaml` file is included for kubernetes deployment, using `traefik` as ingress, but it is easily customizable to other ingresses.

Just fix the secret, which will contain the mattermost hook URL, and the hostname.