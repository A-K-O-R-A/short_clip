# Short clip
This simple selfhosted server lets you upload your clipboard contents and places a link in your clipbaord that you can share with others

## Setup

### Server

In the working directory of the server create a `.authorized_tokens` file with the following format:

```
token1 username1
token2 username2
```

Then simply run the server, the port can be changed by setting the `PORT` environment variable.

### Client

Create a config file at `$HOME/.config/shortclip-config.json` on linux or `%APPDATA%/shortclip-config.json` on windows.

```json
{
  "token": "yourtoken",
  "host": "https://your.domain.com"
}

```

The client can now be run either as daemon or oneshot application. It is recommended to set up the shortcut through your desktop environment instead of using the daemon but both should work.

Now simply copy something to your clipboard and press your predefined shortcut