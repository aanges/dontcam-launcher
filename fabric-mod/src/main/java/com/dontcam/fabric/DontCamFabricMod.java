package com.dontcam.fabric;

import com.dontcam.fabric.gui.ClientOptionsScreen;
import com.dontcam.fabric.hud.ClickTracker;
import com.dontcam.fabric.hud.HudOverlay;
import com.dontcam.fabric.net.DontCamNetworking;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;

import java.util.Collections;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public class DontCamFabricMod implements ClientModInitializer {

    public static final String MODID = "dontcam";
    public static final String VERSION = "0.1.0";

    /** Owner crown UUID. Crown renders only for Microsoft-authenticated sessions. */
    public static final UUID OWNER_UUID = UUID.fromString("76b1ef92-f047-428a-a068-ccd4d1eb4863");

    public static KeyBinding openMenu;
    public static ClientConfig config;

    /** UUIDs confirmed to run DontCam (received via roster channel). */
    public static final Set<UUID> roster = Collections.synchronizedSet(new HashSet<>());
    /** Per-UUID Microsoft flag from the roster (owner crown needs microsoft=true). */
    public static final Map<UUID, Boolean> rosterMicrosoft = new ConcurrentHashMap<>();

    public static boolean localMicrosoft = false;
    public static UUID localUuid = null;

    @Override
    public void onInitializeClient() {
        config = new ClientConfig(MinecraftClient.getInstance().runDirectory.toPath().resolve("config").toFile());

        localMicrosoft = "msa".equalsIgnoreCase(System.getProperty("dontcam.auth", ""));
        try {
            String raw = System.getProperty("dontcam.uuid", "");
            if (!raw.isEmpty()) {
                localUuid = UUID.fromString(raw);
            }
        } catch (IllegalArgumentException ignored) {
            localUuid = null;
        }

        openMenu = KeyBindingHelper.registerKeyBinding(
                new KeyBinding("key.dontcam.menu", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_RIGHT_SHIFT, "category.dontcam"));

        DontCamNetworking.init();
        HudOverlay.init();
        ClickTracker.init();

        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            if (client.player == null || client.currentScreen != null) {
                return;
            }
            while (openMenu.wasPressed()) {
                client.setScreen(new ClientOptionsScreen());
            }
        });
    }

    /** True when the given player is known to run DontCam (self or roster). */
    public static boolean isDontCam(UUID uuid) {
        if (uuid == null) {
            return false;
        }
        if (uuid.equals(localUuid)) {
            return true;
        }
        return roster.contains(uuid);
    }

    /** Owner crown: matching UUID AND a Microsoft session (offline never counts). */
    public static boolean isOwner(UUID uuid) {
        if (uuid == null || !OWNER_UUID.equals(uuid)) {
            return false;
        }
        if (uuid.equals(localUuid)) {
            return localMicrosoft;
        }
        Boolean ms = rosterMicrosoft.get(uuid);
        return ms != null && ms;
    }
}
