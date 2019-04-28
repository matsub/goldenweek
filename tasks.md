STUN on UDP

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
(STUNの実装者にはあんま関係ない)

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
        2. the value of RTO SHOULD NOT be rounded up to the nearest second.  Rather, a 1 ms accuracy SHOULD be maintained
    - The value for RTO SHOULD be cached by a client after the completion of the transaction
        - and used as the starting value for RTO for the next transaction to the same server (based on equality of IP address).
        - The value SHOULD be considered stale and discarded after 10 minutes.
- Retransmissions continue until a response is received, or until a total of Rc requests have been sent.
    - Rc SHOULD be configurable and SHOULD have a default of 7.
- If, after the last request, a duration equal to Rm times the RTO has passed without a response the client SHOULD consider the transaction to have failed.
    - Rm SHOULD be configurable and SHOULD have a default of 16.
- A STUN transaction over UDP is also considered failed if there has been a hard ICMP error.
    - e.g.  when RTO = 500 ms
        - requests would be sent at times 0 ms, 500 ms, 1500 ms, 3500 ms, 7500 ms, 15500 ms, and 31500 ms.
        - If the client has not received a response after 39500 ms
        - the client will consider the transaction to have timed out.

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


# Basic Server Behavior
- The STUN server MUST support the Binding method.
- It SHOULD NOT utilize the short-term or long-term credential mechanism.
- It SHOULD NOT utilize the ALTERNATE-SERVER mechanism.
- (TCPは許して) It MUST support UDP and TCP.
- It MAY support STUN over TCP/TLS; however, TLS provides minimal security benefits in this basic mode of operation.
- It MAY utilize the FINGERPRINT mechanism but MUST NOT require it.
    - Since the stand-alone server only runs STUN, FINGERPRINT provides no benefit.
    - Requiring it would break compatibility with RFC 3489, and such compatibility is desirable in a stand-alone server.
- (無視する) Stand-alone STUN servers SHOULD support backwards compatibility with [RFC3489] clients, as described in Section 12.

- (あっハイ) It is RECOMMENDED that administrators of STUN servers provide DNS entries for those servers as described in Section 9.
- A basic STUN server is not a solution for NAT traversal by itself.
    - However, it can be utilized as part of a solution through STUN usages.

## でも
- the STUN server functionality in an agent supporting connectivity checks would utilize short-term credentials.


# Short-Term Credential Mechanism
- For a request or indication message, the agent MUST include the USERNAME and MESSAGE-INTEGRITY attributes in the message.


# STUN Attributes
これを読みまくるしかない
https://tools.ietf.org/html/rfc5389#section-15
