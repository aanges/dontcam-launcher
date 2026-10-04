package com.dontcam.fabric;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;

/**
 * Per-module HUD configuration stored in {@code config/dontcam.json}.
 *
 * <p>Each module has: enabled, x/y position, scale, text color, background
 * toggle and label toggle. Old flat configs ({@code fps/cps/keystrokes}
 * booleans) are migrated automatically. The DC tag is always rendered and
 * is intentionally NOT part of this config.
 */
public class ClientConfig {

    public static class Module {
        public boolean enabled = true;
        public int x = 5;
        public int y = 5;
        public float scale = 1.0f;
        public int color = 0xFFFFFF;
        /** "static" | "chroma" | "rainbow" (animated modes ignore {@link #color}). */
        public String colorMode = "static";
        public boolean bg = true;
        public boolean label = true;
    }

    public Module fps = new Module();
    public Module cps = new Module();
    public Module keys = new Module();

    private final File file;

    public ClientConfig(File configDir) {
        if (!configDir.exists()) {
            configDir.mkdirs();
        }
        this.file = new File(configDir, "dontcam.json");
        defaults();
        load();
    }

    private void defaults() {
        fps.x = 5;
        fps.y = 5;
        cps.x = 5;
        cps.y = 23;
        // Keys: x/y are offsets added to the auto (bottom-center) anchor.
        keys.x = 0;
        keys.y = 0;
    }

    private static void readModule(Module m, JsonObject o) {
        if (o == null) {
            return;
        }
        if (o.has("enabled")) {
            m.enabled = o.get("enabled").getAsBoolean();
        }
        if (o.has("x")) {
            m.x = o.get("x").getAsInt();
        }
        if (o.has("y")) {
            m.y = o.get("y").getAsInt();
        }
        if (o.has("scale")) {
            try {
                m.scale = Math.min(3.0f, Math.max(0.5f, (float) o.get("scale").getAsDouble()));
            } catch (Exception ignored) {
            }
        }
        if (o.has("color")) {
            try {
                m.color = o.get("color").getAsInt() & 0xFFFFFF;
            } catch (Exception ignored) {
            }
        }
        if (o.has("colorMode")) {
            try {
                String mode = o.get("colorMode").getAsString();
                m.colorMode = ("chroma".equals(mode) || "rainbow".equals(mode)) ? mode : "static";
            } catch (Exception ignored) {
            }
        }
        if (o.has("bg")) {
            m.bg = o.get("bg").getAsBoolean();
        }
        if (o.has("label")) {
            m.label = o.get("label").getAsBoolean();
        }
    }

    public void load() {
        if (!file.exists()) {
            save();
            return;
        }
        try (FileReader reader = new FileReader(file)) {
            JsonObject json = JsonParser.parseReader(reader).getAsJsonObject();
            if (json.has("modules")) {
                JsonObject mods = json.getAsJsonObject("modules");
                readModule(fps, mods.has("fps") ? mods.getAsJsonObject("fps") : null);
                readModule(cps, mods.has("cps") ? mods.getAsJsonObject("cps") : null);
                readModule(keys, mods.has("keys") ? mods.getAsJsonObject("keys") : null);
            } else {
                // Migrate legacy flat toggles.
                if (json.has("fps")) {
                    fps.enabled = json.get("fps").getAsBoolean();
                }
                if (json.has("cps")) {
                    cps.enabled = json.get("cps").getAsBoolean();
                }
                if (json.has("keystrokes")) {
                    keys.enabled = json.get("keystrokes").getAsBoolean();
                }
                save();
            }
        } catch (Exception ignored) {
        }
    }

    private static JsonObject writeModule(Module m) {
        JsonObject o = new JsonObject();
        o.addProperty("enabled", m.enabled);
        o.addProperty("x", m.x);
        o.addProperty("y", m.y);
        o.addProperty("scale", m.scale);
        o.addProperty("color", m.color);
        o.addProperty("colorMode", m.colorMode == null ? "static" : m.colorMode);
        o.addProperty("bg", m.bg);
        o.addProperty("label", m.label);
        return o;
    }

    public void save() {
        try (FileWriter writer = new FileWriter(file)) {
            JsonObject mods = new JsonObject();
            mods.add("fps", writeModule(fps));
            mods.add("cps", writeModule(cps));
            mods.add("keys", writeModule(keys));
            JsonObject root = new JsonObject();
            root.add("modules", mods);
            writer.write(root.toString());
        } catch (Exception ignored) {
        }
    }

    public void resetLayout() {
        defaults();
        fps.scale = 1.0f;
        cps.scale = 1.0f;
        keys.scale = 1.0f;
        save();
    }
}
