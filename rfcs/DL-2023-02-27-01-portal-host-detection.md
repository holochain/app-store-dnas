
# RFC - Portal Host Detection

Feb 25, 2023


## Question

What is the best user experience we can create within the next month?


## Context

When an Agent is completely offline, it will take the full Holochain timeout to get a response.
Holochain currently supports a vector of calls so that a list of potential hosts can be queried all
at once.  This still results in the full wait for the timeout if any of the hosts are offline.

This is because QUIC does not differentiate between the connection timeout and the request timeout.
So this problem should go away with WebRTC.


## Options

### Vector Call that doesn't require all responses

App Store would be able to call a vector of "hosts" and specify that only 1 response is required.
Holochain would then call all the hosts simultaneously and return the first response it gets back.

Pros

- Responses would be as quick as possible

Cons

- Requires Holochain team to implement new feature
- Information about non-responsive hosts is thrown away

#### Previous Discussions

- https://chat.holochain.org/holo/pl/zttj615wp7bwuezpc8yma8cjjc


### Ping/Pong Vector Call

App Store would call a vector of "hosts" with a ping/pong (like a heartbeat) to check who is
available.  Once all hosts have responded, or reached the timeout, App Store can use an available
host for the install request.

Pros

- Can be implemented quickly without Holochain team

Cons

- The minimum wait time to install an app would be the timeout + time get and install the package


## Decision

TBD
