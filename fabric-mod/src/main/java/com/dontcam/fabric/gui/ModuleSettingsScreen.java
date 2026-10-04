package com.dontcam.fabric.gui;

import com.dontcam.fabric.ClientConfig;
import com.dontcam.fabric.DontCamFabricMod;
import com.dontcam.fabric.hud.HudOverlay;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/**
 * Per-module settings: position (sliders + drag on the live preview),
 * scale, text color, background and label toggles.
 */
public class ModuleSettingsScreen extends Screen {

    private final String id;
    private String dragging;
    private int colorRowY;

    public ModuleSettingsScreen(String id) {
        super(Text.literal("Module settings"));
        this.id = id;
    }

    private ClientConfig.Module module() {
        ClientConfig cfg = DontCamFabricMod.config;
        if ("cps".equals(id)) {
            return cfg.cps;
        }
        if ("keys".equals(id)) {
            return cfg.keys;
        }
        return cfg.fps;
    }

    private String title() {
        if ("cps".equals(id)) {
            return "CPS";
        }
        if ("keys".equals(id)) {
            return "Keystrokes";
        }
        return "FPS";
    }

    private void stepper(String label, String value, Runnable dec, Runnable inc, int y) {
        int cx = this.width / 2;
        this.addDrawableChild(new DontCamButtonWidget(cx - 110, y, 24, 20,
                Text.literal("<"), b -> dec.run()));
        this.addDrawableChild(new DontCamButtonWidget(cx + 86, y, 24, 20,
                Text.literal(">"), b -> inc.run()));
        rowLabels.add(new String[]{label + ": " + value, String.valueOf(cx), String.valueOf(y + 6)});
    }

    private final java.util.List<String[]> rowLabels = new java.util.ArrayList<>();

    @Override
    protected void init() {
        rowLabels.clear();
        this.clearChildren();
        ClientConfig.Module m = module();
        int y = this.height / 2 - 52;

        stepper("X", String.valueOf(m.x), () -> {
            m.x -= 5;
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, () -> {
            m.x += 5;
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, y);
        stepper("Y", String.valueOf(m.y), () -> {
            m.y -= 5;
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, () -> {
            m.y += 5;
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, y + 24);
        stepper("Size", "x" + Math.round(m.scale * 100) / 100.0, () -> {
            m.scale = Math.max(0.5f, m.scale - 0.25f);
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, () -> {
            m.scale = Math.min(3.0f, m.scale + 0.25f);
            DontCamFabricMod.config.save();
            this.clearAndInit();
        }, y + 48);

        int cx = this.width / 2;
        colorRowY = y + 72;
        this.addDrawableChild(new DontCamButtonWidget(cx - 86, colorRowY, 196, 20,
                Text.literal("Change color..."),
                b -> {
                    DontCamFabricMod.config.save();
                    if (this.client != null) {
                        this.client.setScreen(new ColorPickerScreen(id));
                    }
                }));
        if (!"keys".equals(id)) {
            this.addDrawableChild(new DontCamButtonWidget(cx - 110, y + 96, 108, 20,
                    Text.literal("Label: " + onOff(m.label)),
                    b -> {
                        m.label = !m.label;
                        DontCamFabricMod.config.save();
                        this.clearAndInit();
                    }));
            this.addDrawableChild(new DontCamButtonWidget(cx + 2, y + 96, 108, 20,
                    Text.literal("Back: " + onOff(m.bg)),
                    b -> {
                        m.bg = !m.bg;
                        DontCamFabricMod.config.save();
                        this.clearAndInit();
                    }));
        } else {
            this.addDrawableChild(new DontCamButtonWidget(cx - 110, y + 96, 220, 20,
                    Text.literal("Back: " + onOff(m.bg)),
                    b -> {
                        m.bg = !m.bg;
                        DontCamFabricMod.config.save();
                        this.clearAndInit();
                    }));
        }
        this.addDrawableChild(new DontCamButtonWidget(cx - 110, y + 120, 220, 20,
                Text.literal("Back"),
                b -> {
                    DontCamFabricMod.config.save();
                    if (this.client != null) {
                        this.client.setScreen(new ClientOptionsScreen());
                    }
                }));
    }

    private static String onOff(boolean v) {
        return v ? "\u00A7aON" : "\u00A7cOFF";
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (button == 0) {
            int[] r = HudOverlay.lastRects.get(id);
            if (r != null && mouseX >= r[0] && mouseX <= r[0] + r[2]
                    && mouseY >= r[1] && mouseY <= r[1] + r[3]) {
                dragging = id;
                dragOffX = (int) mouseX - r[0];
                dragOffY = (int) mouseY - r[1];
                return true;
            }
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    private int dragOffX;
    private int dragOffY;

    @Override
    public boolean mouseDragged(double mouseX, double mouseY, int button, double dx, double dy) {
        if (dragging != null && button == 0) {
            ClientConfig.Module m = module();
            if ("keys".equals(id)) {
                int[] an = HudOverlay.keysAnchor(this.width, this.height, previewModuleAtDrag(m));
                m.x = (int) Math.round(mouseX - dragOffX - (an[0] - m.x));
                m.y = (int) Math.round(mouseY - dragOffY - (an[1] - m.y));
            } else {
                m.x = (int) Math.round(mouseX - dragOffX);
                m.y = (int) Math.round(mouseY - dragOffY);
            }
            m.x = Math.max(-400, Math.min(this.width + 400, m.x));
            m.y = Math.max(-400, Math.min(this.height + 400, m.y));
            return true;
        }
        return super.mouseDragged(mouseX, mouseY, button, dx, dy);
    }

    /** Anchor computed without the current offsets (they are being replaced). */
    private ClientConfig.Module previewModuleAtDrag(ClientConfig.Module m) {
        ClientConfig.Module copy = new ClientConfig.Module();
        copy.enabled = m.enabled;
        copy.x = 0;
        copy.y = 0;
        copy.scale = m.scale;
        copy.color = m.color;
        copy.bg = m.bg;
        copy.label = m.label;
        return copy;
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        if (dragging != null && button == 0) {
            dragging = null;
            DontCamFabricMod.config.save();
            return true;
        }
        return super.mouseReleased(mouseX, mouseY, button);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, this.width, this.height, 0x80000000);
        // Live preview behind the panel (also refreshes drag rects).
        HudOverlay.drawPreview(context, this.width, this.height, this.textRenderer);
        context.drawCenteredTextWithShadow(this.textRenderer, title() + " settings",
                this.width / 2, this.height / 2 - 78, 0xFFFFFFFF);
        // Color swatch next to the Change button.
        int sw = HudOverlay.resolveColor(module());
        int cx = this.width / 2;
        context.fill(cx - 110, colorRowY, cx - 90, colorRowY + 20, 0xFF101018);
        context.fill(cx - 109, colorRowY + 1, cx - 91, colorRowY + 19, sw);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal("drag the module on screen to move it").styled(s -> s.withColor(0xFF7DD3FC)),
                this.width / 2, this.height / 2 + 96, 0xFFFFFFFF);
        super.render(context, mouseX, mouseY, delta);
        for (String[] row : rowLabels) {
            int rcx = Integer.parseInt(row[1]);
            int ry = Integer.parseInt(row[2]);
            int tw = this.textRenderer.getWidth(row[0]);
            context.drawText(this.textRenderer, row[0], rcx - tw / 2, ry, 0xFFFFFFFF, false);
        }
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
