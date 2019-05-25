# RFC5389 自分用まとめ
RFC5389実装するときの情報まとめ。元ネタ: https://tools.ietf.org/html/rfc5389

# Header Structure
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|0 0|     STUN Message Type     |         Message Length        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Magic Cookie                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                     Transaction ID (96 bits)                  |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

## STUN message Type
```
 0                 1
 2  3  4 5 6 7 8 9 0 1 2 3 4 5
+--+--+-+-+-+-+-+-+-+-+-+-+-+-+
|M |M |M|M|M|C|M|M|M|C|M|M|M|M|
|11|10|9|8|7|1|6|5|4|0|3|2|1|0|
+--+--+-+-+-+-+-+-+-+-+-+-+-+-+
```

- the bits in the message type field are shown as most significant (M11) through least significant (M0).
- C1 and C0 represent a 2-bit encoding of the class.
    - A class of 0b00 is a request,
    - a class of 0b01 is an indication,
    - a class of 0b10 is a success response,
    - and a class of 0b11 is an error response.

## Magic Cookie
`0x2112A442`

## Transaction ID
96-bit identifier

- For request/response transactions, the transaction ID is chosen by the STUN client for the request and echoed by the server in the response.
- For indications, it is chosen by the agent sending the indication.
- As such, the transaction ID MUST be uniformly and randomly chosen from the interval 0 .. 2**96-1, and SHOULD be cryptographically random.
- Resends of the same request reuse the same transaction ID,
    - but the client MUST choose a new transaction ID for new transactions
        - unless the new request is bit-wise identical to the previous request
        - and sent from the same transport address to the same IP address.
- Success and error responses MUST carry the same transaction ID as their corresponding request.
- When an agent is acting as a STUN server and STUN client on the same port
    - the transaction IDs in requests sent by the agent have no relationship to the transaction IDs in requests received by the agent.

## Message Length
- The message length MUST contain the size, in bytes, of the messag
    - not including the 20-byte STUN header. 
- Since all STUN attributes are padded to a multiple of 4 bytes, the last 2 bits of this field are always zero.


#  Base Protocol Procedures

## Forming a Request or an Indication
- If the agent is sending a request, it SHOULD add a SOFTWARE attribute to the request.
- Agents MAY include a SOFTWARE attribute in indications, depending on the method.
- For the Binding method with no authentication, no attributes are required unless the usage specifies otherwise.
- All STUN messages sent over UDP SHOULD be less than the path MTU, if known.
- STUN provides no ability to handle the case where the request is under the MTU but the response would be larger than the MTU.

## Sending the Request or Indication
The STUN usage must specify which transport protocol is used, and how the agent determines the IP address and port of the recipient.

## Sending over UDP
- Reliability of STUN request/ response transactions is accomplished through retransmissions of the request message by the client application itself.
- STUN indications are not retransmitted; thus, indication transactions over UDP are not reliable.
- A client SHOULD retransmit a STUN request message starting with an interval of RTO ("Retransmission TimeOut")
    - doubling after each retransmission.
    -  The RTO is an estimate of the round-trip time (RTT), and is computed as described in RFC 2988, with two exceptions.
        1. the initial value for RTO SHOULD be configurable (rather than the 3 s recommended in RFC 2988) and SHOULD be greater than 500 ms.
        2. the value of RTO SHOULD NOT be rounded up to the nearest second. Rather, a 1 ms accuracy SHOULD be maintained
    - The value for RTO SHOULD be cached by a client after the completion of the transaction
        - and used as the starting value for RTO for the next transaction to the same server (based on equality of IP address).
        - The value SHOULD be considered stale and discarded after 10 minutes.
- Retransmissions continue until a response is received, or until a total of Rc requests have been sent.
    - Rc SHOULD be configurable and SHOULD have a default of 7.
- If, after the last request, a duration equal to Rm times the RTO has passed without a response the client SHOULD consider the transaction to have failed.
    - Rm SHOULD be configurable and SHOULD have a default of 16.
- A STUN transaction over UDP is also considered failed if there has been a hard ICMP error.
    - e.g. when RTO = 500 ms
        - requests would be sent at times 0 ms, 500 ms, 1500 ms, 3500 ms, 7500 ms, 15500 ms, and 31500 ms.
        - If the client has not received a response after 39500 ms
        - the client will consider the transaction to have timed out.

## Sending over TCP or TLS-over-TCP
- In some usages of STUN, STUN is sent as the only protocol over the TCP connection.
    - In this case, it can be sent without the aid of any additional framing or demultiplexing.
    - In other usages, or with other extensions, it may be multiplexed with other data over a TCP connection.
- The STUN service running on the well-known port or ports discovered through the DNS procedures is for STUN alone, and not for STUN multiplexed with other data.
- When additional framing is utilized, the usage will specify how the client knows to apply it and what port to connect to.
- When STUN is run by itself over TLS-over-TCP,
    - the TLS_RSA_WITH_AES_128_CBC_SHA ciphersuite MUST be implemented at a minimum.
    - Implementations MAY also support any other ciphersuite.
- When it receives the TLS Certificate message,
    - the client SHOULD verify the certificate and inspect the site identified by the certificate.
- If the certificate is invalid or revoked, or if it does not identify the appropriate party,
    - the client MUST NOT send the STUN message or otherwise proceed with the STUN transaction.
- The client MUST verify the identity of the server.
    - To do that, it follows the identification procedures defined in [Section 3.1 of RFC 2818](https://tools.ietf.org/html/rfc2818#section-3.1).
- Those procedures assume the client is dereferencing a URI.
    - the client treats the domain name or IP address used in Section 8.1 as the host portion of the URI that has been dereferenced.
    - Alternatively, a client MAY be configured with a set of domains or IP addresses that are trusted;
        - if a certificate is received that identifies one of those domains or IP addresses,
            - the client considers the identity of the server to be verified.
- When STUN is run multiplexed with other protocols over a TLS-over-TCP connection,
    - the mandatory ciphersuites and TLS handling procedures operate as defined by those protocols.
- Reliability of STUN over TCP and TLS-over-TCP is handled by TCP itself, and there are no retransmissions at the STUN protocol level.
    - However, for a request/response transaction,
        - if the client has not received a response by Ti seconds after it sent the SYN to establish the connection,
        - it considers the transaction to have timed out.
        - Ti SHOULD be configurable and SHOULD have a default of 39.5s.
            - This value has been chosen to equalize the TCP and UDP timeouts for the default initial RTO.
    - In addition, if the client is unable to establish the TCP connection,
        - or the TCP connection is reset or fails before a response is received,
        - any request/response transaction in progress is considered to have failed.
- The client MAY send multiple transactions over a single TCP (or TLS- over-TCP) connection,
    - and it MAY send another request before receiving a response to the previous.
- The client SHOULD keep the connection open until it:
    - has no further STUN requests or indications to send over that connection,
    - has no plans to use any resources (such as a mapped address (MAPPED-ADDRESS or XOR-MAPPED-ADDRESS) or relayed address [BEHAVE-TURN]) that were learned though STUN requests sent over that connection,
    - if multiplexing other application protocols over that port,
        - has finished using that other application,
    - if using that learned port with a remote peer,
        - has established communications with that remote peer, as is required by some TCP NAT traversal techniques.
- At the server end,
    - the server SHOULD keep the connection open,
    - and let the client close it,
    - unless the server has determined that the connection has timed out (for example, due to the client disconnecting from the network).
- Bindings learned by the client will remain valid in intervening NATs only while the connection remains open.
- Only the client knows how long it needs the binding.
- The server SHOULD NOT close a connection if a request was received over that connection for which a response was not sent.
- A server MUST NOT ever open a connection back towards the client in order to send a response.
- Servers SHOULD follow best practices regarding connection management in cases of overload.

## Receiving a STUN Message
When a STUN agent receives a STUN message, it first checks that the message obeys the rules of Section 6.

- It checks that
    - the first two bits are 0
    - the magic cookie field has the correct value
    - the message length is sensible
    - and that the method value is a supported method.
- If the message class is "Success Response" or "Error Response"
    - the agent checks that the transaction ID matches a transaction that is still in progress.
- If the FINGERPRINT extension is being used, the agent checks that
    - the FINGERPRINT attribute is present and contains the correct value.
-  If any errors are detected, the message is silently discarded.
- Unknown comprehension-optional attributes MUST be ignored by the agent.
- Known-but-unexpected attributes SHOULD be ignored by the agent.

## Processing a Request
-  If the request contains one or more unknown comprehension-required attributes
    - the server replies with an error response with an error code of 420 (Unknown Attribute)
    - and includes an UNKNOWN-ATTRIBUTES attribute in the response that lists the unknown comprehension-required attributes.
- The server then does any additional checking that the method or the specific usage requires.
    - If all the checks succeed, the server formulates a success response as described below.

- When UDP, a request received by the server could be the first request of a transaction, or a retransmission.
- The server MUST respond to retransmissions such that the following property is preserved:
    - if the client receives the response to the retransmission and not the response that was sent to the original request,
        - the overall state on the client and server is identical to the case where only the response to the original retransmission is received,
        - or where both responses are received (in which case the client will use the first).
        - The easiest way to meet this requirement is for the server to remember all transaction IDs received over UDP and their corresponding responses in the last 40 seconds.
            - However, this requires the server to hold state, and will be inappropriate for any requests which are not authenticated.
        - Another way is to reprocess the request and recompute the response.
        - The latter technique MUST only be applied to requests that are idempotent (a request is considered idempotent when the same request can be safely repeated without impacting the overall state of the system) and result in the same success response for the same request.
        - The Binding method is considered to be idempotent.
        - Note that there are certain rare network events that could cause the reflexive transport address value to change, resulting in a different mapped address in different success responses.
        - Extensions to STUN MUST discuss the implications of request retransmissions on servers that do not store transaction state.


### Forming a Success or Error Response
- For an error response, the server MUST add an ERROR-CODE attribute
    - The reason phrase is not fixed, but SHOULD be something suitable for the error code.
    - For certain errors, additional attributes are added to the message. (spelled out in the description).
    - Extensions may define other errors and/or additional attributes to add in error cases.
    - When forming the success response, the server adds a XOR-MAPPED-ADDRESS attribute to the response, where the contents of the attribute are the source transport address of the request message.
        - For UDP, this is the source IP address and source UDP port of the request message.
        - For TCP and TLS-over-TCP, this is the source IP address and source TCP port of the TCP connection as seen by the server.


## Processing an Indication
- If the indication contains unknown comprehension-required attributes,
    - the indication is discarded and processing ceases.
    - The agent then does any additional checking that the method or the specific usage requires.
- If all the checks succeed, the agent then processes the indication.
- No response is generated for an indication.
- For the Binding method, no additional checking or processing is required, unless the usage specifies otherwise.
- Since indications are not re-transmitted over UDP (unlike requests), there is no need to handle re-transmissions of indications at the sending agent.

### Processing a Success Response
- If the success response contains unknown comprehension-required attributes,
    - the response is discarded and the transaction is considered to have failed.
    - The client then does any additional checking that the method or the specific usage requires.
- If all the checks succeed, the client then processes the success response.
- For the Binding method, the client checks that the XOR-MAPPED-ADDRESS attribute is present in the response.
    - The client checks the address family specified.
    - If it is an unsupported address family, the attribute SHOULD be ignored.
    - If it is an unexpected but supported address family
        - (for example, the Binding transaction was sent over IPv4, but the address family specified is IPv6),
        - then the client MAY accept and use the value.

### Processing an Error Response
- If the error response contains unknown comprehension-required attributes,
    - or if the error response does not contain an ERROR-CODE attribute,
        - then the transaction is simply considered to have failed.
        - The client then does any processing specified by the authentication mechanism.
- The processing at this point depends on the error code, the method, and the usage; the following are the default rules:

#### Default Rules with Processing an Error Response
- If the error code is 300 through 399,
    - the client SHOULD consider the transaction as failed unless the ALTERNATE-SERVER extension is being used.
- If the error code is 400 through 499,
    - the client declares the transaction failed;
    - in the case of 420 (Unknown Attribute),
        - the response should contain a UNKNOWN-ATTRIBUTES attribute that gives additional information.
- If the error code is 500 through 599,
    - the client MAY resend the request;
    - clients that do so MUST limit the number of times they do this.
- Any other error code causes the client to consider the transaction failed.

# Basic Server Behavior
- The STUN server MUST support the Binding method.
- It SHOULD NOT utilize the short-term or long-term credential mechanism.
- It SHOULD NOT utilize the ALTERNATE-SERVER mechanism.
- It MUST support UDP and TCP.
- It MAY support STUN over TCP/TLS; however, TLS provides minimal security benefits in this basic mode of operation.
- It MAY utilize the FINGERPRINT mechanism but MUST NOT require it.
    - Since the stand-alone server only runs STUN, FINGERPRINT provides no benefit.
    - Requiring it would break compatibility with RFC 3489, and such compatibility is desirable in a stand-alone server.
- Stand-alone STUN servers SHOULD support backwards compatibility with [RFC3489] clients, as described in Section 12.

- It is RECOMMENDED that administrators of STUN servers provide DNS entries for those servers as described in Section 9.
- A basic STUN server is not a solution for NAT traversal by itself.
    - However, it can be utilized as part of a solution through STUN usages.
- the STUN server functionality in an agent supporting connectivity checks would utilize short-term credentials.


# [WIP] DNS Discovery of a Server


# Short-Term Credential Mechanism
- For a request or indication message, the agent MUST include the USERNAME and MESSAGE-INTEGRITY attributes in the message.

## Receiving a Request or Indication
- If the message does not contain both a MESSAGE-INTEGRITY and a USERNAME attribute:
    - If the message is a request, the server MUST reject the request with an error response (an error code of 400).
    - If the message is an indication, the agent MUST silently discard the indication.
- If the USERNAME does not contain a username value currently valid within the server:
    - If the message is a request, the server MUST reject the request with an error response (an error code of 401).
    - If the message is an indication, the agent MUST silently discard the indication.
- If the resulting value does not match the contents of the MESSAGE-INTEGRITY attribute:
    - If the message is a request, the server MUST reject the request with an error response (an error code of 401).
    - If the message is an indication, the agent MUST silently discard the indication.
- Any response generated by a server MUST include the MESSAGE-INTEGRITY attribute.
- The response MUST NOT contain the USERNAME attribute.
- If any of the checks fail, a server MUST NOT include a MESSAGE-INTEGRITY or USERNAME attribute in the error response.

## Receiving a Response
- The client looks for the MESSAGE-INTEGRITY attribute in the response.
    - If present, the client computes the message integrity over the response as defined in Section 15.4, using the same password it utilized for the request.
    - If the resulting value matches the contents of the MESSAGE-INTEGRITY attribute,
        - the response is considered authenticated.
    - If the value does not match, or if MESSAGE-INTEGRITY was absent,
        - the response MUST be discarded, as if it was never received.
        - This means that retransmits, if applicable, will continue.

# [WIP] Long-Term Credential Mechanism

# [WIP] STUN Attributes

# [WIP] Security Considerations
