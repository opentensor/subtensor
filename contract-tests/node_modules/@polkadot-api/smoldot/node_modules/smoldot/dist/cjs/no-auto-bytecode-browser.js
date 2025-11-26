"use strict";
// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.startWithBytecode = exports.QueueFullError = exports.JsonRpcDisabledError = exports.CrashError = exports.AlreadyDestroyedError = exports.AddChainError = void 0;
const client_js_1 = require("./internals/client.js");
var public_types_js_1 = require("./public-types.js");
Object.defineProperty(exports, "AddChainError", { enumerable: true, get: function () { return public_types_js_1.AddChainError; } });
Object.defineProperty(exports, "AlreadyDestroyedError", { enumerable: true, get: function () { return public_types_js_1.AlreadyDestroyedError; } });
Object.defineProperty(exports, "CrashError", { enumerable: true, get: function () { return public_types_js_1.CrashError; } });
Object.defineProperty(exports, "JsonRpcDisabledError", { enumerable: true, get: function () { return public_types_js_1.JsonRpcDisabledError; } });
Object.defineProperty(exports, "QueueFullError", { enumerable: true, get: function () { return public_types_js_1.QueueFullError; } });
/**
 * Initializes a new client. This is a pre-requisite to connecting to a blockchain.
 *
 * Can never fail.
 *
 * @param options Configuration of the client.
 */
function startWithBytecode(options) {
    options.forbidTcp = true;
    // When in a secure context, browsers refuse to open non-secure WebSocket connections to
    // non-localhost. There is an exception if the page is localhost, in which case all connections
    // are allowed.
    // Detecting this ahead of time is better for the overall health of the client, as it will
    // avoid storing in memory addresses that it knows it can't connect to.
    // The condition below is a hint, and false-positives or false-negatives are not fundamentally
    // an issue.
    if ((typeof isSecureContext === 'boolean' && isSecureContext) && typeof location !== undefined) {
        const loc = location.toString();
        if (loc.indexOf('localhost') !== -1 && loc.indexOf('127.0.0.1') !== -1 && loc.indexOf('::1') !== -1) {
            options.forbidNonLocalWs = true;
        }
    }
    return (0, client_js_1.start)(options, options.bytecode, {
        performanceNow: () => {
            return performance.now();
        },
        getRandomValues: (buffer) => {
            const crypto = globalThis.crypto;
            if (!crypto)
                throw new Error('randomness not available');
            // Browsers have this completely undocumented behavior (it's not even part of a spec)
            // that for some reason `getRandomValues` can't be called on arrayviews back by
            // `SharedArrayBuffer`s and they throw an exception if you try.
            if (buffer.buffer instanceof ArrayBuffer)
                crypto.getRandomValues(buffer);
            else {
                const tmpArray = new Uint8Array(buffer.length);
                crypto.getRandomValues(tmpArray);
                buffer.set(tmpArray);
            }
        },
        connect: (config) => {
            return connect(config);
        }
    });
}
exports.startWithBytecode = startWithBytecode;
/**
 * Tries to open a new connection using the given configuration.
 *
 * @see Connection
 * @throws any If the multiaddress couldn't be parsed or contains an invalid protocol.
 */
function connect(config) {
    if (config.address.ty === "websocket") {
        // Even though the WHATWG specification (<https://websockets.spec.whatwg.org/#dom-websocket-websocket>)
        // doesn't mention it, `new WebSocket` can throw an exception if the URL is forbidden
        // for security reasons. We absord this exception as soon as it is thrown.
        // `connection` can be either a `WebSocket` object (the normal case), or a string
        // indicating an error message that must be propagated with `onConnectionReset` as soon
        // as possible, or `null` if the API user considers the connection as reset.
        let connection;
        try {
            connection = new WebSocket(config.address.url);
        }
        catch (error) {
            connection = error instanceof Error ? error.toString() : "Exception thrown by new WebSocket";
        }
        const bufferedAmountCheck = { quenedUnreportedBytes: 0, nextTimeout: 10 };
        const checkBufferedAmount = () => {
            if (!(connection instanceof WebSocket))
                return;
            if (connection.readyState != 1)
                return;
            // Note that we might expect `bufferedAmount` to always be <= the sum of the lengths
            // of all the data that has been sent, but that might not be the case. For this
            // reason, we use `bufferedAmount` as a hint rather than a correct value.
            const bufferedAmount = connection.bufferedAmount;
            let wasSent = bufferedAmountCheck.quenedUnreportedBytes - bufferedAmount;
            if (wasSent < 0)
                wasSent = 0;
            bufferedAmountCheck.quenedUnreportedBytes -= wasSent;
            if (bufferedAmountCheck.quenedUnreportedBytes != 0) {
                setTimeout(checkBufferedAmount, bufferedAmountCheck.nextTimeout);
                bufferedAmountCheck.nextTimeout *= 2;
                if (bufferedAmountCheck.nextTimeout > 500)
                    bufferedAmountCheck.nextTimeout = 500;
            }
            // Note: it is important to call `onWritableBytes` at the very end, as it might
            // trigger a call to `send`.
            if (wasSent != 0)
                config.onWritableBytes(wasSent);
        };
        if (connection instanceof WebSocket) {
            connection.binaryType = 'arraybuffer';
            connection.onopen = () => {
                config.onWritableBytes(1024 * 1024);
            };
            connection.onclose = (event) => {
                const message = "Error code " + event.code + (!!event.reason ? (": " + event.reason) : "");
                config.onConnectionReset(message);
            };
            connection.onmessage = (msg) => {
                config.onMessage(new Uint8Array(msg.data));
            };
        }
        else {
            setTimeout(() => {
                if (connection && !(connection instanceof WebSocket)) {
                    config.onConnectionReset(connection);
                    connection = null;
                }
            }, 1);
        }
        return {
            reset: () => {
                if (connection instanceof WebSocket) {
                    connection.onopen = null;
                    connection.onclose = null;
                    connection.onmessage = null;
                    connection.onerror = null;
                    // According to the WebSocket specification, calling `close()` when a WebSocket
                    // isn't fully opened yet is completely legal and seemingly a normal thing to
                    // do (see <https://websockets.spec.whatwg.org/#dom-websocket-close>).
                    // Unfortunately, browsers print a warning in the console if you do that. To
                    // avoid these warnings, we only call `close()` if the connection is fully
                    // opened. According to <https://websockets.spec.whatwg.org/#garbage-collection>,
                    // removing all the event listeners will cause the WebSocket to be garbage
                    // collected, which should have the same effect as `close()`.
                    if (connection.readyState == WebSocket.OPEN)
                        connection.close();
                }
                connection = null;
            },
            send: (data) => {
                if (bufferedAmountCheck.quenedUnreportedBytes == 0) {
                    bufferedAmountCheck.nextTimeout = 10;
                    setTimeout(checkBufferedAmount, 10);
                }
                for (const buffer of data) {
                    bufferedAmountCheck.quenedUnreportedBytes += buffer.length;
                }
                connection.send(new Blob(data));
            },
            closeSend: () => { throw new Error('Wrong connection type'); },
            openOutSubstream: () => { throw new Error('Wrong connection type'); }
        };
    }
    else if (config.address.ty === "webrtc") {
        const { targetPort, ipVersion, targetIp, remoteTlsCertificateSha256 } = config.address;
        const state = {
            pc: undefined,
            dataChannels: new Map(),
            nextStreamId: 0,
            isFirstOutSubstream: true,
        };
        // Kills all the JavaScript objects (the connection and all its substreams), ensuring that no
        // callback will be called again. Doesn't report anything to smoldot, as this should be done
        // by the caller.
        const killAllJs = () => {
            // The `RTCPeerConnection` is created pretty quickly. It is however still possible for
            // smoldot to cancel the opening, in which case `pc` will still be undefined.
            if (!state.pc) {
                console.assert(state.dataChannels.size === 0, "substreams exist while pc is undef");
                state.pc = null;
                return;
            }
            state.pc.onconnectionstatechange = null;
            state.pc.onnegotiationneeded = null;
            state.pc.ondatachannel = null;
            for (const channel of Array.from(state.dataChannels.values())) {
                channel.channel.onopen = null;
                channel.channel.onerror = null;
                channel.channel.onclose = null;
                channel.channel.onbufferedamountlow = null;
                channel.channel.onmessage = null;
            }
            state.dataChannels.clear();
            state.pc.close(); // Not necessarily necessary, but it doesn't hurt to do so.
        };
        // Function that configures a newly-opened channel and adds it to the map. Used for both
        // inbound and outbound substreams.
        const addChannel = (dataChannel, direction) => {
            const streamId = state.nextStreamId;
            state.nextStreamId += 1;
            dataChannel.binaryType = 'arraybuffer';
            let isOpen = { value: false };
            dataChannel.onopen = () => {
                console.assert(!isOpen.value, "substream opened twice");
                isOpen.value = true;
                config.onStreamOpened(streamId, direction);
                config.onWritableBytes(65536, streamId);
            };
            dataChannel.onerror = dataChannel.onclose = (event) => {
                // Note that Firefox doesn't support <https://developer.mozilla.org/en-US/docs/Web/API/RTCErrorEvent>.
                const message = (event instanceof RTCErrorEvent) ? event.error.toString() : "RTCDataChannel closed";
                if (!isOpen.value) {
                    // Substream wasn't opened yet and thus has failed to open. The API has no
                    // mechanism to report substream openings failures. We could try opening it
                    // again, but given that it's unlikely to succeed, we simply opt to kill the
                    // entire connection.
                    killAllJs();
                    // Note that the event doesn't give any additional reason for the failure.
                    config.onConnectionReset("data channel failed to open: " + message);
                }
                else {
                    // Substream was open and is now closed. Normal situation.
                    dataChannel.onopen = null;
                    dataChannel.onerror = null;
                    dataChannel.onclose = null;
                    dataChannel.onbufferedamountlow = null;
                    dataChannel.onmessage = null;
                    state.dataChannels.delete(streamId);
                    config.onStreamReset(streamId, message);
                }
            };
            dataChannel.onbufferedamountlow = () => {
                const channel = state.dataChannels.get(streamId);
                const val = channel.bufferedBytes;
                channel.bufferedBytes = 0;
                config.onWritableBytes(val, streamId);
            };
            dataChannel.onmessage = (m) => {
                // The `data` field is an `ArrayBuffer`.
                config.onMessage(new Uint8Array(m.data), streamId);
            };
            state.dataChannels.set(streamId, { channel: dataChannel, bufferedBytes: 0 });
        };
        // It is possible for the browser to use multiple different certificates.
        // In order for our local certificate to be deterministic, we need to generate it manually and
        // set it explicitly as part of the configuration.
        // According to <https://w3c.github.io/webrtc-pc/#dom-rtcpeerconnection-generatecertificate>,
        // browsers are guaranteed to support `{ name: "ECDSA", namedCurve: "P-256" }`.
        RTCPeerConnection.generateCertificate({ name: "ECDSA", namedCurve: "P-256", hash: "SHA-256" }).then((localCertificate) => __awaiter(this, void 0, void 0, function* () {
            if (state.pc === null)
                return;
            // Due to <https://bugzilla.mozilla.org/show_bug.cgi?id=1659672>, connections from
            // Firefox to a localhost WebRTC server always fails. Since this bug has been opened
            // for three years at the time of writing, it is unlikely to be fixed in the short
            // term. In order to provider better user feedback, we straight up refuse connecting
            // and stop the connection.
            // Note that this is just a hint. Failing to detect this will lead to the WebRTC
            // handshake  timing out.
            // TODO: eventually remove this if the Firefox bug is fixed
            if ((targetIp == 'localhost' || targetIp == '127.0.0.1' || targetIp == '::1') && navigator.userAgent.indexOf('Firefox') !== -1) {
                killAllJs();
                config.onConnectionReset("Firefox can't connect to a localhost WebRTC server");
                return;
            }
            // Create a new WebRTC connection.
            state.pc = new RTCPeerConnection({ certificates: [localCertificate] });
            // We need to build the multihash corresponding to the local certificate.
            // While there exists a `RTCPeerConnection.getFingerprints` function, Firefox notably
            // doesn't support it.
            // See <https://developer.mozilla.org/en-US/docs/Web/API/RTCCertificate#browser_compatibility>
            // An alternative to `getFingerprints` is to ask the browser to generate an SDP offer and
            // extract from fingerprint from it. Because we explicitly provide a certificate, we have
            // the guarantee that the list of certificates will always be the same whenever an SDP offer
            // is generated by the browser. However, while this alternative does work on Firefox, it
            // doesn't on Chrome, as the SDP offer is for some reason missing the fingerprints.
            // Therefore, our strategy is to use `getFingerprints` when it is available (i.e. every
            // browser except Firefox), and parse the SDP offer when it is not (i.e. Firefox). In the
            // future, only `getFingerprints` would be used.
            let localTlsCertificateHex;
            if (localCertificate.getFingerprints) {
                for (const { algorithm, value } of localCertificate.getFingerprints()) {
                    if (algorithm === 'sha-256') {
                        localTlsCertificateHex = value;
                        break;
                    }
                }
            }
            else {
                const localSdpOffer = yield state.pc.createOffer();
                // Note that this regex is not strict. The browser isn't a malicious actor, and the
                // objective of this regex is not to detect invalid input.
                const localSdpOfferFingerprintMatch = localSdpOffer.sdp.match(/a(\s*)=(\s*)fingerprint:(\s*)(sha|SHA)-256(\s*)(([a-fA-F0-9]{2}(:)*){32})/);
                if (localSdpOfferFingerprintMatch) {
                    localTlsCertificateHex = localSdpOfferFingerprintMatch[6];
                }
            }
            if (localTlsCertificateHex === undefined) {
                // Because we've already returned from the `connect` function at this point, we pretend
                // that the connection has failed to open.
                config.onConnectionReset('Failed to obtain the browser certificate fingerprint');
                return;
            }
            let localTlsCertificateSha256 = new Uint8Array(32);
            localTlsCertificateSha256.set(localTlsCertificateHex.split(':').map((s) => parseInt(s, 16)), 0);
            // `onconnectionstatechange` is used to detect when the connection has closed or has failed
            // to open.
            // Note that smoldot will think that the connection is open even when it is still opening.
            // Therefore we don't care about events concerning the fact that the connection is now fully
            // open.
            state.pc.onconnectionstatechange = (_event) => {
                if (state.pc.connectionState == "closed" || state.pc.connectionState == "disconnected" || state.pc.connectionState == "failed") {
                    killAllJs();
                    config.onConnectionReset("WebRTC state transitioned to " + state.pc.connectionState);
                }
            };
            state.pc.onnegotiationneeded = (_event) => __awaiter(this, void 0, void 0, function* () {
                var _a;
                // Create a new offer and set it as local description.
                let sdpOffer = (yield state.pc.createOffer()).sdp;
                // We check that the locally-generated SDP offer has a data channel with the UDP
                // protocol. If that isn't the case, the connection will likely fail.
                if (sdpOffer.match(/^m=application(\s+)(\d+)(\s+)UDP\/DTLS\/SCTP(\s+)webrtc-datachannel$/m) === null) {
                    console.error("Local offer doesn't contain UDP data channel. WebRTC connections will likely fail. Please report this issue.");
                }
                // According to the libp2p WebRTC spec, the ufrag and pwd are the same
                // randomly-generated string on both sides, and must be prefixed with
                // `libp2p-webrtc-v1:`. We modify the local description to ensure that.
                // While we could randomly generate a new string, we just grab the one that the
                // browser has generated, in order to make sure that it respects the constraints
                // of the ICE protocol.
                const browserGeneratedPwd = (_a = sdpOffer.match(/^a=ice-pwd:(.+)$/m)) === null || _a === void 0 ? void 0 : _a.at(1);
                if (browserGeneratedPwd === undefined) {
                    console.error("Failed to set ufrag to pwd. WebRTC connections will likely fail. Please report this issue.");
                }
                const ufragPwd = "libp2p+webrtc+v1/" + browserGeneratedPwd;
                sdpOffer = sdpOffer.replace(/^a=ice-ufrag.*$/m, 'a=ice-ufrag:' + ufragPwd);
                sdpOffer = sdpOffer.replace(/^a=ice-pwd.*$/m, 'a=ice-pwd:' + ufragPwd);
                yield state.pc.setLocalDescription({ type: 'offer', sdp: sdpOffer });
                // Transform certificate hash into fingerprint (upper-hex; each byte separated by ":").
                const fingerprint = Array.from(remoteTlsCertificateSha256).map((n) => ("0" + n.toString(16)).slice(-2).toUpperCase()).join(':');
                // Note that the trailing line feed is important, as otherwise Chrome
                // fails to parse the payload.
                const remoteSdp = 
                // Version of the SDP protocol. Always 0. (RFC8866)
                "v=0" + "\n" +
                    // Identifies the creator of the SDP document. We are allowed to use dummy values
                    // (`-` and `0.0.0.0`) to remain anonymous, which we do. Note that "IN" means
                    // "Internet" (and not "input"). (RFC8866)
                    "o=- 0 0 IN IP" + ipVersion + " " + targetIp + "\n" +
                    // Name for the session. We are allowed to pass a dummy `-`. (RFC8866)
                    "s=-" + "\n" +
                    // Start and end of the validity of the session. `0 0` means that the session never
                    // expires. (RFC8866)
                    "t=0 0" + "\n" +
                    // A lite implementation is only appropriate for devices that will
                    // always be connected to the public Internet and have a public
                    // IP address at which it can receive packets from any
                    // correspondent.  ICE will not function when a lite implementation
                    // is placed behind a NAT (RFC8445).
                    "a=ice-lite" + "\n" +
                    // A `m=` line describes a request to establish a certain protocol.
                    // The protocol in this line (i.e. `TCP/DTLS/SCTP` or `UDP/DTLS/SCTP`) must always be
                    // the same as the one in the offer. We know that this is true because checked above.
                    // The `<fmt>` component must always be `webrtc-datachannel` for WebRTC.
                    // The rest of the SDP payload adds attributes to this specific media stream.
                    // RFCs: 8839, 8866, 8841
                    "m=application " + String(targetPort) + " " + "UDP/DTLS/SCTP webrtc-datachannel" + "\n" +
                    // Indicates the IP address of the remote.
                    // Note that "IN" means "Internet" (and not "input").
                    "c=IN IP" + ipVersion + " " + targetIp + "\n" +
                    // Media ID - uniquely identifies this media stream (RFC9143).
                    "a=mid:0" + "\n" +
                    // Indicates that we are complying with RFC8839 (as oppposed to the legacy RFC5245).
                    "a=ice-options:ice2" + "\n" +
                    // ICE username and password, which are used for establishing and
                    // maintaining the ICE connection. (RFC8839)
                    // These values are set according to the libp2p WebRTC specification.
                    "a=ice-ufrag:" + ufragPwd + "\n" +
                    "a=ice-pwd:" + ufragPwd + "\n" +
                    // Fingerprint of the certificate that the server will use during the TLS
                    // handshake. (RFC8122)
                    // MUST be derived from the certificate used by the answerer (server).
                    "a=fingerprint:sha-256 " + fingerprint + "\n" +
                    // Indicates that the remote DTLS server will only listen for incoming
                    // connections. (RFC5763)
                    // The answerer (server) MUST not be located behind a NAT (RFC6135).
                    "a=setup:passive" + "\n" +
                    // The SCTP port (RFC8841)
                    // Note it's different from the "m=" line port value, which
                    // indicates the port of the underlying transport-layer protocol
                    // (UDP or TCP)
                    "a=sctp-port:5000" + "\n" +
                    // The maximum SCTP user message size (in bytes) (RFC8841)
                    // Setting this field is part of the libp2p spec.
                    "a=max-message-size:16384" + "\n" +
                    // A transport address for a candidate that can be used for connectivity
                    // checks (RFC8839).
                    "a=candidate:1 1 UDP 1 " + targetIp + " " + String(targetPort) + " typ host" + "\n";
                yield state.pc.setRemoteDescription({ type: "answer", sdp: remoteSdp });
            });
            state.pc.ondatachannel = ({ channel }) => {
                // TODO: is the substream maybe already open? according to the Internet it seems that no but it's unclear
                addChannel(channel, 'inbound');
            };
            config.onMultistreamHandshakeInfo({
                handshake: 'webrtc',
                localTlsCertificateSha256,
            });
        }));
        return {
            reset: (streamId) => {
                // If `streamId` is undefined, then the whole connection must be destroyed.
                if (streamId === undefined) {
                    killAllJs();
                }
                else {
                    const channel = state.dataChannels.get(streamId);
                    channel.channel.onopen = null;
                    channel.channel.onerror = null;
                    channel.channel.onclose = null;
                    channel.channel.onbufferedamountlow = null;
                    channel.channel.onmessage = null;
                    channel.channel.close();
                    state.dataChannels.delete(streamId);
                }
            },
            send: (data, streamId) => {
                const channel = state.dataChannels.get(streamId);
                for (const buffer of data) {
                    channel.bufferedBytes += buffer.length;
                }
                channel.channel.send(new Blob(data));
            },
            closeSend: () => { throw new Error('Wrong connection type'); },
            openOutSubstream: () => {
                // `openOutSubstream` can only be called after we have called `config.onOpen`,
                // therefore `pc` is guaranteed to be non-null.
                // Note that the label passed to `createDataChannel` is required to be empty as
                // per the libp2p WebRTC specification.
                // TODO: adjusting the options based on the first substream is a bit hacky
                const opts = state.isFirstOutSubstream ? { negotiated: true, id: 0 } : {};
                state.isFirstOutSubstream = false;
                addChannel(state.pc.createDataChannel("", opts), 'outbound');
            }
        };
    }
    else {
        // Should never happen, as we tweak the options to refuse connection types that
        // we don't support.
        throw new Error();
    }
}
