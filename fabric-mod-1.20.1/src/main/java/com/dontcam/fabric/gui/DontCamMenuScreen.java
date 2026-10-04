package com.dontcam.fabric.gui;

import com.dontcam.fabric.DontCamFabricMod;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Right-Shift menu: toggle the DontCam modules. */
public class DontCamMenuScreen extends Screen {

    public DontCamMenuScreen() {
        super(Text.literal("DontCam Client"));
    }

    private void addToggle(String label, boolean value, java.util.function.Consumer<Boolean> setter, int y) {
        int cx = this.width / 2;
        String state = value ? "§aON" : "§cOFF";
        this.addDrawableChild(new DontCamButtonWidget(cx - 100, y, 200, 20,
                Text.literal(label + ": " + state),
                button -> {
                    setter.accept(!value);
                    DontCamFabricMod.config.save();
                    this.clearAndInit();
                }));
    }

    @Override
    protected void init() {
        int y = this.height / 2 - 44;
        addToggle("FPS Counter", DontCamFabricMod.config.fps, v -> DontCamFabricMod.config.fps = v, y);
        addToggle("CPS Counter", DontCamFabricMod.config.cps, v -> DontCamFabricMod.config.cps = v, y + 24);
        addToggle("Keystrokes", DontCamFabricMod.config.keystrokes, v -> DontCamFabricMod.config.keystrokes = v, y + 48);
        addToggle("DC Badges", DontCamFabricMod.config.badges, v -> DontCamFabricMod.config.badges = v, y + 72);
        this.addDrawableChild(new DontCamButtonWidget(this.width / 2 - 100, y + 100, 200, 20,
                Text.literal("Done"),
                button -> {
                    DontCamFabricMod.config.save();
                    this.close();
                }));
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context);
        context.drawCenteredTextWithShadow(this.textRenderer, "§bDontCam §9Client",
                this.width / 2, this.height / 2 - 70, 0xFFFFFF);
        String auth = DontCamFabricMod.localMicrosoft ? "§aMicrosoft" : "§7Offline";
        context.drawCenteredTextWithShadow(this.textRenderer, auth,
                this.width / 2, this.height / 2 - 58, 0xFFFFFF);
        super.render(context, mouseX, mouseY, delta);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
