package com.dontcam.fabric.net;

import com.dontcam.fabric.DontCamFabricMod;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import io.netty.buffer.Unpooled;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.Identifier;

import java.nio.charset.StandardCharsets;
import java.util.UUID;

/**
 * Roster sync with a DontCam-aware server (1.20.1-era networking API:
 * raw Identifier + PacketByteBuf, no CustomPayload records).
 *
 * Client announces itself on join; a companion server plugin may answer with a
 * roster payload. Without one, only the local badge/crown render.
 */
public class DontCamNetworking {

    public static final Identifier HELLO = new Identifier("dontcam", "hello");
    public static final Identifier ROSTER = new Identifier("dontcam", "roster");

    public static void init() {
        ClientPlayNetworking.registerGlobalReceiver(ROSTER, (client, handler, buf, responseSender) -> {
            byte[] bytes = new byte[buf.readableBytes()];
            buf.readBytes(bytes);
            String json = new String(bytes, StandardCharsets.UTF_8);
            client.execute(() -> applyRoster(json));
        });

        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> {
            String uuid = DontCamFabricMod.localUuid != null
                    ? DontCamFabricMod.localUuid.toString().replace("-", "")
                    : "";
            String hello = "{\"uuid\":\"" + uuid + "\",\"microsoft\":" + DontCamFabricMod.localMicrosoft
                    + ",\"mod\":\"" + DontCamFabricMod.VERSION + "\"}";
            PacketByteBuf buf = new PacketByteBuf(Unpooled.buffer());
            buf.writeString(hello);
            ClientPlayNetworking.send(HELLO, buf);
        });
    }

    static void applyRoster(String json) {
        try {
            JsonObject root = JsonParser.parseString(json).getAsJsonObject();
            JsonArray players = root.getAsJsonArray("players");
            if (players == null) {
                return;
            }
            DontCamFabricMod.roster.clear();
            DontCamFabricMod.rosterMicrosoft.clear();
            for (JsonElement element : players) {
                if (!element.isJsonObject()) {
                    continue;
                }
                JsonObject player = element.getAsJsonObject();
                if (!player.has("uuid")) {
                    continue;
                }
                try {
                    UUID uuid = UUID.fromString(player.get("uuid").getAsString());
                    boolean microsoft = player.has("microsoft") && player.get("microsoft").getAsBoolean();
                    DontCamFabricMod.roster.add(uuid);
                    DontCamFabricMod.rosterMicrosoft.put(uuid, microsoft);
                } catch (IllegalArgumentException ignored) {
                }
            }
        } catch (Exception ignored) {
        }
    }
}
