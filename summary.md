# Memo of RFC5389 (for myself)
This is a memo for me to implement rfc5389. source is here: https://tools.ietf.org/html/rfc5389

will update WIPs if i feel like it.

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
- After the STUN header are zero or more attributes.
- Each attribute MUST be TLV encoded, with a 16-bit type, 16-bit length, and value. # TLV=Type-length-value
- Each STUN attribute MUST end on a 32-bit boundary.
- As mentioned above, all fields in an attribute are transmitted most significant bit first.

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Type                  |            Length             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Value (variable)                ....
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```


- The value in the length field MUST contain the length of the Value part of the attribute, prior to padding,
    - measured in bytes.
- The padding bits are ignored, and may be any value.
- Any attribute type MAY appear more than once in a STUN message.
- Unless specified otherwise, the order of appearance is significant:
    - only the first occurrence needs to be processed by a receiver,
    - and any duplicates MAY be ignored by a receiver.
- Attributes with type values between 0x0000 and 0x7FFF are comprehension-required attributes,
    - which means that the STUN agent cannot successfully process the message unless it understands the attribute.
- Attributes with type values between 0x8000 and 0xFFFF are comprehension-optional attributes,
    - which means that those attributes can be ignored by the STUN agent if it does not understand them.
- The set of STUN attribute types is maintained by IANA.
- The rest of this section describes the format of the various attributes defined in this specification.

## MAPPED-ADDRESS
- The MAPPED-ADDRESS attribute indicates a reflexive transport address of the client.
- It consists of an 8-bit address family and a 16-bit port

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|0 0 0 0 0 0 0 0|    Family     |           Port                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                 Address (32 bits or 128 bits)                 |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- The address family can take on the following values:
    - `0x01:IPv4`
    - `0x02:IPv6`
- The first 8 bits of the MAPPED-ADDRESS MUST be set to 0 and MUST be ignored by receivers.
- This attribute is used only by servers for achieving backwards compatibility with RFC 3489 clients.

## XOR-MAPPED-ADDRESS
- The XOR-MAPPED-ADDRESS attribute is identical to the MAPPED-ADDRESS attribute,
    - except that the reflexive transport address is obfuscated through the XOR function.

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|x x x x x x x x|    Family     |         X-Port                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                X-Address (Variable)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- The Family represents the IP address family, and is encoded identically to the Family in MAPPED-ADDRESS.
- If the IP address family is IPv4,
    1. X-Address is computed by taking the mapped IP address in host byte order,
    2. XOR'ing it with the magic cookie, and converting the result to network byte order.
- If the IP address family is IPv6,
    1. X-Address is computed by taking the mapped IP address in host byte order,
    2. XOR'ing it with the concatenation of the magic cookie and the 96-bit transaction ID,
    3. and converting the result to network byte order.
- The rules below are the same as for MAPPED-ADDRESS.
    - encoding and processing the first 8 bits of the attribute's value,
    - the rules for handling multiple occurrences of the attribute,
    - and the rules for processing address families

> Note:
XOR-MAPPED-ADDRESS and MAPPED-ADDRESS differ only in their encoding of the transport address.
The former encodes the transport address by exclusive-or'ing it with the magic cookie.
The latter encodes it directly in binary.
RFC 3489 originally specified only MAPPED-ADDRESS.
However, deployment experience found that some NATs rewrite the 32-bit binary payloads containing the NAT's public IP address, such as STUN's MAPPED-ADDRESS attribute, in the well-meaning but misguided attempt at providing a generic ALG function.
Such behavior interferes with the operation of STUN and also causes failure of STUN's message-integrity checking.

## USERNAME
- The USERNAME attribute is used for message integrity.
- It identifies the username and password combination used in the message-integrity check.
- The value of USERNAME is a variable-length value.
- It MUST contain a UTF-8 encoded sequence of less than 513 bytes,
    - and MUST have been processed using [SASLprep](https://tools.ietf.org/html/rfc4013).

## MESSAGE-INTEGRITY
- The MESSAGE-INTEGRITY attribute contains an HMAC-SHA1 of the STUN message.
- The MESSAGE-INTEGRITY attribute can be present in any STUN message type.
- Since it uses the SHA1 hash, the HMAC will be 20 bytes.
- The text used as input to HMAC is the STUN message,
    - including the header, up to and including the attribute preceding the MESSAGE-INTEGRITY attribute.
- With the exception of the FINGERPRINT attribute,
    - which appears after MESSAGE-INTEGRITY,
    - agents MUST ignore all other attributes that follow MESSAGE-INTEGRITY.
- For long-term credentials, the key is 16 bytes:

```
key = MD5(username ":" realm ":" SASLprep(password))
```

- That is, the 16-byte key is formed by taking the MD5 hash of the result of concatenating the following five fields:
    - the username, with any quotes and trailing nulls removed, as taken from the USERNAME attribute
        - (in which case SASLprep has already been applied)
    - a single colon
    - the realm, with any quotes and trailing nulls removed
    - a single colon
    - the password, with any trailing nulls removed and after processing using SASLprep.
- For example, if
    - the username was 'user',
    - the realm was 'realm',
    - password was 'pass'
    - then the 16-byte HMAC key would be the result of performing an MD5 hash on the string `user:realm:pass`,
    - the resulting hash being `0x8493fbc53ba582fb4c044c456bdc40eb`.
- For short-term credentials:

```
key = SASLprep(password)
```

- the hash used to construct MESSAGE-INTEGRITY includes the length field from the STUN message header.
- Prior to performing the hash, the MESSAGE-INTEGRITY attribute MUST be inserted into the message (with dummy content).
- The length MUST then be set to point to the length of the message up to,
    - and including, the MESSAGE-INTEGRITY attribute itself, but excluding any attributes after it.
- Once the computation is performed, the value of the MESSAGE-INTEGRITY attribute can be filled in,
    - and the value of the length in the STUN header can be set to its correct value -- the length of the entire message.
- Similarly, when validating the MESSAGE-INTEGRITY,
    - the length field should be adjusted to point to the end of the MESSAGE-INTEGRITY attribute prior to calculating the HMAC.
- Such adjustment is necessary when attributes, such as FINGERPRINT, appear after MESSAGE-INTEGRITY.

## FINGERPRINT
- The FINGERPRINT attribute MAY be present in all STUN messages.
- The value of the attribute is computed as the CRC-32 of the STUN message up to (but excluding) the FINGERPRINT attribute itself,
    - XOR'ed with the 32-bit value `0x5354554e` (the XOR helps in cases where an application packet is also using CRC-32 in it).
- The 32-bit CRC is the one defined in ITU V.42, which has a generator polynomial of `x32+x26+x23+x22+x16+x12+x11+x10+x8+x7+x5+x4+x2+x+1`.
- When present, the FINGERPRINT attribute MUST be the last attribute in the message, and thus will appear after MESSAGE-INTEGRITY.
- As with MESSAGE-INTEGRITY, the CRC used in the FINGERPRINT attribute covers the length field from the STUN message header.
- Therefore, this value must be correct and include the CRC attribute as part of the message length, prior to computation of the CRC.
- When using the FINGERPRINT attribute in a message,
    1. the attribute is first placed into the message with a dummy value,
    2. then the CRC is computed,
    3. and then the value of the attribute is updated.
- If the MESSAGE-INTEGRITY attribute is also present,
    - then it must be present with the correct message-integrity value before the CRC is computed,
    - since the CRC is done over the value of the MESSAGE-INTEGRITY attribute as well.

## ERROR-CODE
- The ERROR-CODE attribute is used in error response messages.
- It contains a numeric error code value in the range of `300 to 699` plus a textual reason phrase encoded in UTF-8,
    - and is consistent in its code assignments and semantics with SIP and HTTP.
- The reason phrase is meant for user consumption, and can be anything appropriate for the error code.
- Recommended reason phrases for the defined error codes are included in the IANA registry for error codes.
- The reason phrase MUST be a UTF-8 encoded sequence of less than 128 characters (which can be as long as 763 bytes).

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0|Class|     Number    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|      Reason Phrase (variable)                                ..
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

- To facilitate processing, the class of the error code (the hundreds digit) is encoded separately from the rest of the code.
- The Class represents the hundreds digit of the error code. The value MUST be between 3 and 6.
- The Number represents the error code modulo 100, and its value MUST be between 0 and 99.
- The following error codes, along with their recommended reason phrases, are defined:

- `300 Try Alternate`: The client should contact an alternate server for this request.
    - This error response MUST only be sent if the request included a USERNAME attribute and a valid MESSAGE-INTEGRITY attribute
        - otherwise, it MUST NOT be sent and error code 400 (Bad Request) is suggested.
    - This error response MUST be protected with the MESSAGE-INTEGRITY attribute,
        - and receivers MUST validate the MESSAGE-INTEGRITY of this response before redirecting themselves to an alternate server.
    - Note: Failure to generate and validate message integrity for a 300 response allows an on-path attacker to falsify a 300 response thus causing subsequent STUN messages to be sent to a victim.
- `400 Bad Request`: The request was malformed.
    - The client SHOULD NOT retry the request without modification from the previous attempt.
    - The server may not be able to generate a valid MESSAGE-INTEGRITY for this error,
        - so the client MUST NOT expect a valid MESSAGE-INTEGRITY attribute on this response.
- `401 Unauthorized`: The request did not contain the correct credentials to proceed.
    - The client should retry the request with proper credentials.
- `420 Unknown Attribute`: The server received a STUN packet containing a comprehension-required attribute that it did not understand.
    - The server MUST put this unknown attribute in the UNKNOWN-ATTRIBUTE attribute of its error response.
- `438 Stale Nonce`: The NONCE used by the client was no longer valid.
    - The client should retry, using the NONCE provided in the response.
- `500 Server Error`: The server has suffered a temporary error.
    - The client should try again.

## REALM
The REALM attribute may be present in requests and responses.
It contains text that meets the grammar for "realm-value" as described in RFC 3261 [RFC3261] but without the double quotes and their surrounding whitespace.
That is, it is an unquoted realm-value (and is therefore a sequence of qdtext or quoted-pair).
It MUST be a UTF-8 [RFC3629] encoded sequence of less than 128 characters (which can be as long as 763 bytes), and MUST have been processed using SASLprep [RFC4013].
Presence of the REALM attribute in a request indicates that long-term credentials are being used for authentication.
Presence in certain error responses indicates that the server wishes the client to use a long-term credential for authentication.

## NONCE
The NONCE attribute may be present in requests and responses.
It contains a sequence of qdtext or quoted-pair, which are defined in RFC 3261 [RFC3261].
Note that this means that the NONCE attribute will not contain actual quote characters.
See RFC 2617 [RFC2617], Section 4.3, for guidance on selection of nonce values in a server.
It MUST be less than 128 characters (which can be as long as 763 bytes).

## UNKNOWN-ATTRIBUTES
The UNKNOWN-ATTRIBUTES attribute is present only in an error response when the response code in the ERROR-CODE attribute is 420.
The attribute contains a list of 16-bit values, each of which represents an attribute type that was not understood by the server.

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|      Attribute 1 Type           |     Attribute 2 Type        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|      Attribute 3 Type           |     Attribute 4 Type    ...
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

> Note: In [RFC3489], this field was padded to 32 by duplicating the last attribute.
In this version of the specification, the normal padding rules for attributes are used instead.

## SOFTWARE
The SOFTWARE attribute contains a textual description of the software being used by the agent sending the message.
It is used by clients and servers.
Its value SHOULD include manufacturer and version number.
The attribute has no impact on operation of the protocol, and serves only as a tool for diagnostic and debugging purposes.
The value of SOFTWARE is variable length.
It MUST be a UTF-8 [RFC3629] encoded sequence of less than 128 characters (which can be as long as 763 bytes).

## ALTERNATE-SERVER
The alternate server represents an alternate transport address identifying a different STUN server that the STUN client should try.
It is encoded in the same way as MAPPED-ADDRESS, and thus refers to a single server by IP address.
The IP address family MUST be identical to that of the source IP address of the request.

# [WIP] IANA Considerations
## [WIP] STUN Methods Registry
A STUN method is a hex number in the range 0x000 - 0xFFF.  The encoding of STUN method into a STUN message is described in Section 6.

- The initial STUN methods are:
    - 0x000: (Reserved)
    - 0x001: Binding
    - 0x002: (Reserved; was SharedSecret)

STUN methods in the range 0x000 - 0x7FF are assigned by IETF Review [RFC5226].  STUN methods in the range 0x800 - 0xFFF are assigned by Designated Expert [RFC5226].  The responsibility of the expert is to verify that the selected codepoint(s) are not in use and that the request is not for an abnormally large number of codepoints.  Technical review of the extension itself is outside the scope of the designated expert responsibility.

## [WIP] STUN Attribute Registry
A STUN Attribute type is a hex number in the range 0x0000 - 0xFFFF.  STUN attribute types in the range 0x0000 - 0x7FFF are considered comprehension-required; STUN attribute types in the range 0x8000 - 0xFFFF are considered comprehension-optional.  A STUN agent handles unknown comprehension-required and comprehension-optional attributes differently.

- The initial STUN Attributes types are:
    - Comprehension-required range (0x0000-0x7FFF):
        - 0x0000: (Reserved)
        - 0x0001: MAPPED-ADDRESS
        - 0x0002: (Reserved; was RESPONSE-ADDRESS)
        - 0x0003: (Reserved; was CHANGE-ADDRESS)
        - 0x0004: (Reserved; was SOURCE-ADDRESS)
        - 0x0005: (Reserved; was CHANGED-ADDRESS)
        - 0x0006: USERNAME
        - 0x0007: (Reserved; was PASSWORD)
        - 0x0008: MESSAGE-INTEGRITY
        - 0x0009: ERROR-CODE
        - 0x000A: UNKNOWN-ATTRIBUTES
        - 0x000B: (Reserved; was REFLECTED-FROM)
        - 0x0014: REALM
        - 0x0015: NONCE
        - 0x0020: XOR-MAPPED-ADDRESS
    - Comprehension-optional range (0x8000-0xFFFF)
        - 0x8022: SOFTWARE
        - 0x8023: ALTERNATE-SERVER
        - 0x8028: FINGERPRINT

STUN Attribute types in the first half of the comprehension-required range (0x0000 - 0x3FFF) and in the first half of the comprehension- optional range (0x8000 - 0xBFFF) are assigned by IETF Review [RFC5226].  STUN Attribute types in the second half of the comprehension-required range (0x4000 - 0x7FFF) and in the second half of the comprehension-optional range (0xC000 - 0xFFFF) are assigned by Designated Expert [RFC5226].  The responsibility of the expert is to verify that the selected codepoint(s) are not in use, and that the request is not for an abnormally large number of codepoints.  Technical review of the extension itself is outside the scope of the designated expert responsibility.

## [WIP] STUN Error Code Registry
A STUN error code is a number in the range 0 - 699.  STUN error codes are accompanied by a textual reason phrase in UTF-8 [RFC3629] that is intended only for human consumption and can be anything appropriate; this document proposes only suggested values.
STUN error codes are consistent in codepoint assignments and semantics with SIP [RFC3261] and HTTP [RFC2616].  The initial values in this registry are given in Section 15.6.

## [WIP] STUN UDP and TCP Port Numbers
IANA has previously assigned port 3478 for STUN.  This port appears in the IANA registry under the moniker "nat-stun-port".  In order to align the DNS SRV procedures with the registered protocol service, IANA is requested to change the name of protocol assigned to port 3478 from "nat-stun-port" to "stun", and the textual name from "Simple Traversal of UDP Through NAT (STUN)" to "Session Traversal Utilities for NAT", so that the IANA port registry would read:

- stun   3478/tcp   Session Traversal Utilities for NAT (STUN) port
- stun   3478/udp   Session Traversal Utilities for NAT (STUN) port

In addition, IANA has assigned port number 5349 for the "stuns" service, defined over TCP and UDP.  The UDP port is not currently defined; however, it is reserved for future use.

# [WIP] Security Considerations
