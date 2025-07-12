# SuperSTAC

a Python library designed for high-availability satellite imagery retrieval. If one data source lacks the requested imagery, DeepEO seamlessly queries alternative sources until it finds a valid result.


# Dependencies

- pystac-client.


### Configuration YML

Template:

```
catalogs:
  Catalog Name:
    url: Catalog URL
    auth:
      type: bearer
      token: "YOUR_MICROSOFT_PC_TOKEN"
  Catalog Name:
    url: Catalog URL
    type: basic
      username: youruser
      password: yourpass
```
# load from the environment variable - SUPERSTAC_CATALOG_CONFIG

# Todo - auth configuration documentation.

See [.superstac.yml](./superstac/.superstac.yml) for an example configuration file.


# todo

- retries - https://pystac-client.readthedocs.io/en/stable/usage.html#configuring-retry-behavior
- modifier - https://pystac-client.readthedocs.io/en/stable/usage.html#automatically-modifying-results
- refresh
- auth
- ues cases e.g when a catalog is offline - store latency ?
- when a catalog is specified and changed it still works - band matching