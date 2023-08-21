# Core Lightning plugin for LNURL and LNAddress support

You can add the plugin by copying it to CLN's plugin directory or by adding the following line to your config file:

```
plugin=/path/to/clnurl
```

For nix-bitcoin based deployments that would be:

```nix
services.clightning = {
  enable = true;
  extraConfig = ''
    plugin=${clnurl}/bin/clnurl
    clnurl_base_address=https://example.com/lnurl_api/
  '';
}
```

where `clnurl` is defined as follows:

```nix
clnurl = (import
  (
    fetchTarball {
      url = "https://github.com/edolstra/flake-compat/archive/b4a34015c698c7793d592d66adbab377907a2be8.tar.gz";
      sha256 = "sha256:1qc703yg0babixi6wshn5wm2kgl5y1drcswgszh4xxzbrwkk9sv7";
    }
  )
  {
    src = fetchTarball {
      url = "https://github.com/elsirion/clnurl/archive/master.tar.gz";
      sha256 = "sha256:0wnvc2i135sqk2vw95wdv2dl34y0gnq3fw61vnsf6fr20610krv6";
    };
  }
).defaultNix.packages.x86_64-linux.default;
```

## Options
`clnurl` exposes the following config options that can be included in CLN's config file or as command line flags:
* `clnurl_base_address`: Specifies the base URL where the API will be hosted. `clnurl` assumes you are running it behind
  a reverse proxy, so even though it might be hosting the API under `http://localhost/lnurl` it might be reachable via
  `https://example.com/lnurl_api/lnurl`, in which case you'd have to specify `https://example.com/lnurl_api/` as base
  address. **You need to set this. It is also important that the reverse proxy uses HTTPS.**
* `clnurl_listen`: Internal listen address for the LNURL web server, defaults to `127.0.0.1:9876`
* `clnurl_min_sendable`: Min millisatoshi amount clnurl is willing to receive, can not be less than 1 or more than maxSendable. Defaults to `100`.
* `clnurl_max_sendable`: Max millisatoshi amount clnurl is willing to receive. Defaults to `100000000000`
* `clnurl_description`: Description used for all LNURLs, PRs to change that welcome. Defaults to `Gimme money!`
* `clnurl_nostr_pubkey`: Nostr HEX pubkey of zapper

## Reverse proxying

```nix
services.nginx = {
  enable = true;
  recommendedProxySettings = true;
  recommendedTlsSettings = true;
  proxyTimeout = "1d";
  virtualHosts."example.com" = {
    enableACME = true;
    forceSSL = true;
    locations."/lnurl_api/" = {
      proxyPass = "http://127.0.0.1:9876/";
    };
    # If you also want to support LN Addresses you can add single handles like this
    locations."=/.well-known/lnurlp/<you_user_name>" = {
      proxyPass = "http://127.0.0.1:9876/";
      # Just added allow origin since that helped with some nostr web clients
      extraConfig = ''
        add_header Access-Control-Allow-Origin *;
      '';
    };
  };
};

security.acme = {
  acceptTerms = true;
  defaults.email = "foo@bar.com";
};

```

## Contributing
I mostly `clnurl` it so I could play with the cool kids on nostr, PRs welcome, but I'm unlikely to fix bugs myself that
don't annoy me personally. Like the MIT license says: "provided as-is".

If you find `clnurl` useful or just want to test it out in the wild feel free to throw me some sats :P

| Format     | Encoding                                                                                            |
|------------|-----------------------------------------------------------------------------------------------------|
| LNURL QR   | <img src="https://raw.githubusercontent.com/elsirion/clnurl/master/elsirion_lnurl.png" width="200"> |
| LNURL      | `lnurl1dp68gurn8ghj7cn5vvknytnnd9exjmmw9e5k7tmvde6hymzlv9cxjtmvde6hymq64r0pl`                       |
| LN Address | `elsirion@sirion.io`                                                                                |
