This is a simple script to implement Dynamic DNS for a home server, assuming the domain is hosted by GoDaddy.

Create a JSON formatted config file that looks like:
```
{
    "api_url": "https://api.godaddy.com",
    "domain": "yourdomain.com",
    "key": "key123key456key789",
    "secret": "secret12345secret67890"
}
```

Then, after compiling to an executable:
```
./godaddy_ddns path/to/config.json 2>&1 >> out.log
```

To complete the implementation of the DDNS, just install that command in your crontab.
