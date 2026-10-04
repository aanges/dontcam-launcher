package com.dontcam.fabric;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.File;
import java.io.FileReader;
import java.io.FileWriter;

/** Simple JSON toggles stored in config/dontcam.json */
public class DontCamConfig {

    public boolean fps = true;
    public boolean cps = true;
    public boolean keystrokes = true;
    public boolean badges = true;

    private final File file;

    public DontCamConfig(File configDir) {
        if (!configDir.exists()) {
            configDir.mkdirs();
        }
        this.file = new File(configDir, "dontcam.json");
        load();
    }

    public void load() {
        if (!file.exists()) {
            save();
            return;
        }
        try (FileReader reader = new FileReader(file)) {
            JsonObject json = JsonParser.parseReader(reader).getAsJsonObject();
            if (json.has("fps")) {
                fps = json.get("fps").getAsBoolean();
            }
            if (json.has("cps")) {
                cps = json.get("cps").getAsBoolean();
            }
            if (json.has("keystrokes")) {
                keystrokes = json.get("keystrokes").getAsBoolean();
            }
            if (json.has("badges")) {
                badges = json.get("badges").getAsBoolean();
            }
        } catch (Exception ignored) {
        }
    }

    public void save() {
        try (FileWriter writer = new FileWriter(file)) {
            JsonObject json = new JsonObject();
            json.addProperty("fps", fps);
            json.addProperty("cps", cps);
            json.addProperty("keystrokes", keystrokes);
            json.addProperty("badges", badges);
            writer.write(json.toString());
        } catch (Exception ignored) {
        }
    }
}
