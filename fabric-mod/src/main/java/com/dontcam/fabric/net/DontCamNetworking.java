package com.dontcam.fabric.net;

import com.dontcam.fabric.DontCamFabricMod;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;
import net.minecraft.network.codec.PacketCodec;
import net.minecraft.network.codec.PacketCodecs;
import net.minecraft.network.packet.CustomPayload;
import net.minecraft.util.Identifier;

import java.nio.charset.StandardCharsets;
import java.util.UUID;

/**
 * Roster sync with a DontCam-aware server.
 *
 * Client announces itself on join; a companion server plugin may answer with a
 * roster payload. Without one, only the local badge/crown render.
 */
public class DontCamNetworking {

    public record HelloPayload(String json) implements CustomPayload {
        public static final CustomPayload.Id<HelloPayload> ID =
                new CustomPayload.Id<>(Identifier.of("dontcam", "hello"));
        public static final PacketCodec<io.netty.buffer.ByteBuf, HelloPayload> CODEC =
                PacketCodecs.STRING.xmap(HelloPayload::new, HelloPayload::json);

        @Override
        public Id<? extends CustomPayload> getId() {
            return ID;
        }
    }

    public record RosterPayload(String json) implements CustomPayload {
        public static final CustomPayload.Id<RosterPayload> ID =
                new CustomPayload.Id<>(Identifier.of("dontcam", "roster"));
        public static final PacketCodec<io.netty.buffer.ByteBuf, RosterPayload> CODEC =
                PacketCodecs.STRING.xmap(RosterPayload::new, RosterPayload::json);

        @Override
        public Id<? extends CustomPayload> getId() {
            return ID;
        }
    }

    public static void init() {
        PayloadTypeRegistry.playC2S().register(HelloPayload.ID, HelloPayload.CODEC);
        PayloadTypeRegistry.playS2C().register(RosterPayload.ID, RosterPayload.CODEC);

        ClientPlayNetworking.registerGlobalReceiver(RosterPayload.ID, (payload, context) -> {
            context.client().execute(() -> applyRoster(payload.json()));
        });

        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> {
            String uuid = DontCamFabricMod.localUuid != null
                    ? DontCamFabricMod.localUuid.toString().replace("-", "")
                    : "";
            String hello = "{\"uuid\":\"" + uuid + "\",\"microsoft\":" + DontCamFabricMod.localMicrosoft
                    + ",\"mod\":\"" + DontCamFabricMod.VERSION + "\"}";
            sender.sendPacket(new HelloPayload(hello));
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
