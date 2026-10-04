package com.dontcam.fabric.gui;

import com.dontcam.fabric.ClientConfig;
import com.dontcam.fabric.DontCamFabricMod;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.List;

/**
 * Transparent in-world menu: "Client Options" with square module tiles.
 * Left-click a tile toggles the module, right-click opens its settings.
 */
public class ClientOptionsScreen extends Screen {

    private final List<DontCamTileWidget> tiles = new ArrayList<>();

    public ClientOptionsScreen() {
        super(Text.literal("Client Options"));
    }

    private void addTile(String id, String name, ClientConfig.Module module, int x, int y) {
        DontCamTileWidget tile = new DontCamTileWidget(x, y, 64, name, module,
                button -> {
                    module.enabled = !module.enabled;
                    DontCamFabricMod.config.save();
                });
        tiles.add(tile);
        this.addDrawableChild(tile);
    }

    @Override
    protected void init() {
        tiles.clear();
        this.clearChildren();
        int cx = this.width / 2;
        int y = this.height / 2 - 40;
        ClientConfig cfg = DontCamFabricMod.config;
        addTile("fps", "FPS", cfg.fps, cx - 104, y);
        addTile("cps", "CPS", cfg.cps, cx - 32, y);
        addTile("keys", "KEYS", cfg.keys, cx + 40, y);
        this.addDrawableChild(new DontCamButtonWidget(cx - 104, y + 76, 100, 20,
                Text.literal("Reset layout"),
                button -> {
                    DontCamFabricMod.config.resetLayout();
                    this.clearAndInit();
                }));
        this.addDrawableChild(new DontCamButtonWidget(cx + 4, y + 76, 100, 20,
                Text.literal("Done"),
                button -> {
                    DontCamFabricMod.config.save();
                    this.close();
                }));
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (button == 1) {
            for (DontCamTileWidget tile : tiles) {
                if (tile.isMouseOver(mouseX, mouseY)) {
                    String id = tileId(tile);
                    if (id != null && this.client != null) {
                        this.client.setScreen(new ModuleSettingsScreen(id));
                        return true;
                    }
                }
            }
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    private String tileId(DontCamTileWidget tile) {
        ClientConfig cfg = DontCamFabricMod.config;
        int index = tiles.indexOf(tile);
        return index == 0 ? "fps" : index == 1 ? "cps" : index == 2 ? "keys" : null;
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        // Transparent: dim the world instead of the vanilla dirt background.
        context.fill(0, 0, this.width, this.height, 0x80000000);
        context.drawCenteredTextWithShadow(this.textRenderer, "Client Options",
                this.width / 2, this.height / 2 - 66, 0xFFFFFFFF);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal("left-click: on/off   right-click: settings").styled(s -> s.withColor(0xFF7DD3FC)),
                this.width / 2, this.height / 2 + 46, 0xFFFFFFFF);
        String auth = DontCamFabricMod.localMicrosoft ? "\u00A7aMicrosoft" : "\u00A77Offline";
        context.drawCenteredTextWithShadow(this.textRenderer, auth, this.width / 2, this.height / 2 + 58, 0xFFFFFF);
        super.render(context, mouseX, mouseY, delta);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
