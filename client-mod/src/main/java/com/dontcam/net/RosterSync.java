package com.dontcam.net;

import com.dontcam.DontCamMod;
import com.dontcam.DontCamMod;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.network.FMLNetworkEvent;

import java.nio.charset.StandardCharsets;
import java.util.UUID;

/**
 * Roster sync with a DontCam-aware server.
 *
 * The server (companion plugin) may send JSON on {@code DontCam|Roster}:
 * <pre>{"players":[{"uuid":"...","microsoft":true}]}</pre>
 * Without a companion plugin only the local player's own badge renders —
 * vanilla servers simply ignore the hello packet.
 */
public class RosterSync {

    @SubscribeEvent
    public void onCustomPacket(FMLNetworkEvent.ClientCustomPacketEvent event) {
        if (event.packet == null) {
            return;
        }
        String channel;
        try {
            channel = event.packet.channel();
        } catch (Exception e) {
            return;
        }
        if (!DontCamMod.CHANNEL_ROSTER.equals(channel)) {
            return;
        }
        try {
            byte[] bytes = new byte[event.packet.payload().readableBytes()];
            event.packet.payload().readBytes(bytes);
            String json = new String(bytes, StandardCharsets.UTF_8);
            JsonObject root = new JsonParser().parse(json).getAsJsonObject();
            JsonArray players = root.getAsJsonArray("players");
            if (players == null) {
                return;
            }
            DontCamMod.roster.clear();
            DontCamMod.rosterMicrosoft.clear();
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
                    DontCamMod.roster.add(uuid);
                    DontCamMod.rosterMicrosoft.put(uuid, microsoft);
                } catch (IllegalArgumentException ignored) {
                }
            }
        } catch (Exception ignored) {
        }
    }
}
