# Netpurr API Client

Netpurr is an open-source, cross-platform API client for REST (Support for other protocols will be gradually added).
It is very compact and swift, coded in Rust.

With Netpurr you can:

* **Debug APIs** using the most popular protocols and formats.
* **Test APIs** using `JavaScript`.
* **Build CI/CD** pipelines using the native `Netpurr` CLI for linting and testing.(Coming soon)
* **Design APIs** using the native OpenAPI editor and visual preview.(Planned support)
* **Mock APIs** (Planned support)
* **Collaborate with others** using the `git` to share your projects.

The following storage options are supported for your projects, collections, specs and all other files:

* **Workspace** Switch between multiple workspaces easily, isolating them from each other.
* **Git Sync** The workspace will support Git synchronization, and file storage will be organized at the granularity of
  APIs, reducing the potential for conflicts during modifications.
* **No remote server** storage involved, ensuring the security of the data.

Performance:

* Extremely fast startup speed, nearly zero opening delay.
* Due to the separate storage of files at the granularity of APIs, changes result in lower disk and memory usage.
* Rust brings excellent memory control and runtime safety.

And a lot more!

* Support for importing Postman data. We have plans to continue supporting data import from Insomnia.
* Real-time rendering of environment variables.
* Introduced `deno-core` as the JavaScript runtime, full support for ES6.

![view.png](pics%2Fview.png)

## Get started

The project is actively in development, with many features continuously being added. You can download the latest builds
from the releases section.

https://github.com/tmtbe/netpurr/releases

There is currently no official 1.0 version. We will release version 1.0 once all foundational features are stable and
ready.

The GitHub automated build will generate two versions: one for `Mac` and one for `Windows`. You can also manually
download the code and compile it yourself.

## Bugs and Feature Requests

Have a bug or a feature request? First, read the issue guidelines and search for existing and closed issues. If your
problem or idea is not addressed yet, please open a new issue.
