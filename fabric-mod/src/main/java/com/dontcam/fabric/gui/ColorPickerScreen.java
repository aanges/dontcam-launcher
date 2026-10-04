package com.dontcam.fabric.gui;

import com.dontcam.fabric.ClientConfig;
import com.dontcam.fabric.DontCamFabricMod;
import com.dontcam.fabric.hud.HudOverlay;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

/**
 * Per-module color picker: big swatch + R/G/B sliders + HEX field +
 * preset palette + Static/Chroma/Rainbow modes. Everything is live.
 */
public class ColorPickerScreen extends Screen {

    private static final int[] PRESETS = {
            0xFFFFFF, 0x55FFFF, 0x55FF55, 0xFFFF55,
            0xFF5555, 0x5555FF, 0xFF55FF, 0xAAAAAA,
    };

    private final String id;
    private DontCamSliderWidget sliderR;
    private DontCamSliderWidget sliderG;
    private DontCamSliderWidget sliderB;
    private TextFieldWidget hexField;

    public ColorPickerScreen(String id) {
        super(Text.literal("Color picker"));
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
            return "CPS color";
        }
        if ("keys".equals(id)) {
            return "Keystrokes color";
        }
        return "FPS color";
    }

    private static int red(int rgb) {
        return (rgb >> 16) & 0xFF;
    }

    private static int green(int rgb) {
        return (rgb >> 8) & 0xFF;
    }

    private static int blue(int rgb) {
        return rgb & 0xFF;
    }

    private static String hex(int rgb) {
        return String.format("%06X", rgb & 0xFFFFFF);
    }

    private void refreshHex() {
        if (hexField != null) {
            hexField.setText(hex(module().color));
        }
    }

    @Override
    protected void init() {
        this.clearChildren();
        ClientConfig.Module m = module();
        int cx = this.width / 2;
        int y = this.height / 2 - 72;

        sliderR = new DontCamSliderWidget(cx - 110, y + 44, 220, "R", red(m.color), v -> {
            m.color = (m.color & 0x00FFFF) | (sliderR.getChannel() << 16);
            refreshHex();
        });
        sliderG = new DontCamSliderWidget(cx - 110, y + 68, 220, "G", green(m.color), v -> {
            m.color = (m.color & 0xFF00FF) | (sliderG.getChannel() << 8);
            refreshHex();
        });
        sliderB = new DontCamSliderWidget(cx - 110, y + 92, 220, "B", blue(m.color), v -> {
            m.color = (m.color & 0xFFFF00) | sliderB.getChannel();
            refreshHex();
        });
        this.addDrawableChild(sliderR);
        this.addDrawableChild(sliderG);
        this.addDrawableChild(sliderB);

        hexField = new TextFieldWidget(this.textRenderer, cx - 110, y + 116, 106, 20, Text.literal("HEX"));
        hexField.setMaxLength(7);
        hexField.setText(hex(m.color));
        hexField.setChangedListener(text -> {
            String t = text.startsWith("#") ? text.substring(1) : text;
            if (t.length() == 6) {
                try {
                    int rgb = Integer.parseInt(t, 16) & 0xFFFFFF;
                    m.color = rgb;
                    sliderR.setChannel(red(rgb));
                    sliderG.setChannel(green(rgb));
                    sliderB.setChannel(blue(rgb));
                } catch (NumberFormatException ignored) {
                }
            }
        });
        this.addDrawableChild(hexField);

        this.addDrawableChild(new DontCamButtonWidget(cx + 2, y + 116, 108, 20,
                Text.literal("Use HEX"),
                b -> refreshHex()));

        // Preset palette row (8 swatches are drawn in render; click zones handled below).
        this.addDrawableChild(new DontCamButtonWidget(cx - 110, y + 140, 68, 20,
                Text.literal("Static"), b -> setMode("static")));
        this.addDrawableChild(new DontCamButtonWidget(cx - 34, y + 140, 68, 20,
                Text.literal("Chroma"), b -> setMode("chroma")));
        this.addDrawableChild(new DontCamButtonWidget(cx + 42, y + 140, 68, 20,
                Text.literal("Rainbow"), b -> setMode("rainbow")));

        this.addDrawableChild(new DontCamButtonWidget(cx - 110, y + 164, 220, 20,
                Text.literal("Back"),
                b -> {
                    DontCamFabricMod.config.save();
                    if (this.client != null) {
                        this.client.setScreen(new ModuleSettingsScreen(id));
                    }
                }));
    }

    private void setMode(String mode) {
        module().colorMode = mode;
        DontCamFabricMod.config.save();
        this.clearAndInit();
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (button == 0) {
            // Preset swatch click zones (drawn in render below the sliders).
            int cx = this.width / 2;
            int y = this.height / 2 - 72;
            for (int i = 0; i < PRESETS.length; i++) {
                int px = cx - 110 + i * 28;
                int py = y + 116 - 24;
                if (mouseX >= px && mouseX < px + 24 && mouseY >= py && mouseY < py + 20) {
                    module().color = PRESETS[i];
                    module().colorMode = "static";
                    DontCamFabricMod.config.save();
                    sliderR.setChannel(red(PRESETS[i]));
                    sliderG.setChannel(green(PRESETS[i]));
                    sliderB.setChannel(blue(PRESETS[i]));
                    refreshHex();
                    return true;
                }
            }
        }
        return super.mouseClicked(mouseX, mouseY, button);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, this.width, this.height, 0x80000000);
        ClientConfig.Module m = module();
        int cx = this.width / 2;
        int y = this.height / 2 - 72;

        context.drawCenteredTextWithShadow(this.textRenderer, title(), cx, y - 24, 0xFFFFFFFF);

        // Big live swatch (animated for chroma/rainbow).
        int argb = HudOverlay.resolveColor(m);
        context.fill(cx - 110, y, cx + 110, y + 36, 0xFF101018);
        context.fill(cx - 108, y + 2, cx + 108, y + 34, argb);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal(modeLabel(m)).styled(s -> s.withColor(0xFF7DD3FC)), cx, y + 12, 0xFFFFFFFF);

        // Preset swatches above the HEX row.
        for (int i = 0; i < PRESETS.length; i++) {
            int px = cx - 110 + i * 28;
            int py = y + 116 - 24;
            context.fill(px, py, px + 24, py + 20, 0xFF000000 | PRESETS[i]);
            context.fill(px, py, px + 24, py + 1, 0xFF2E9BFF);
            context.fill(px, py + 19, px + 24, py + 20, 0xFF2E9BFF);
        }

        super.render(context, mouseX, mouseY, delta);
    }

    private static String modeLabel(ClientConfig.Module m) {
        if ("chroma".equals(m.colorMode)) {
            return "CHROMA";
        }
        if ("rainbow".equals(m.colorMode)) {
            return "RAINBOW";
        }
        return "STATIC";
    }

    @Override
    public void removed() {
        DontCamFabricMod.config.save();
        super.removed();
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
