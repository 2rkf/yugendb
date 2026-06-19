# Driver contract

The driver contract checks that each backend follows the same yugendb storage model:

```text
namespace + collection + key -> serialised typed value
```

Bundled drivers are expected to support set, get, exists, delete, prefix scan, batch writes, missing-value behaviour, and structured errors according to their reported capabilities.
