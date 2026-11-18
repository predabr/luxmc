## Private LAN login and shutdown correction

Version remains 3.0.2. No GitHub release or push.

The login adapter traversed fields of IntegratedServer before recognizing its MinecraftServer superclass. Its bounded reflection search could exhaust all 32 objects before reaching the actual connection. The regression reproduces the old failure with an integrated server containing a populated object graph. Server subclasses are now excluded before traversing their fields.

The Minecraft 26.3 shutdown crash report showed DestroyJavaVM waiting while the Luxmc HUD kept Swing's event queue alive. The agent now waits for the game main thread to finish, stops the HUD timer, and disposes its own windows on the Swing event queue.

Verified:

- Original regression failed with the previous agent and passes with the correction.
- Private connection scope retains official authentication for unrelated ports, external peers, and expired rooms.
- Minecraft 26.3 actual IntegratedServer, ServerLoginPacketListenerImpl, Connection and real Netty TCP socket recognize only the private room port.
- A copy of an existing Minecraft 26.3 world opened in LAN. A raw Minecraft protocol client logged in with a local profile while server authentication was enabled. Received login success, not an encryption request.
- The same real Minecraft process closed with exit code 0, without the shutdown watchdog.
- Full client agent exits cleanly under Java 8, 21 and 25.
- cargo check passed with four existing warnings.

Real Minecraft evidence: C:/Users/preda/AppData/Local/Temp/luxmc-world-login-j0vQDa/test-output.log.

This is local integration validation. A fresh end-to-end test between the user's and cousin's computers is still required to confirm their exact versions, loaders and network conditions.
