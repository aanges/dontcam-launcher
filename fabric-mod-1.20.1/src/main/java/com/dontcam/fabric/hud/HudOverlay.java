package com.dontcam.fabric.hud;

import com.dontcam.fabric.DontCamFabricMod;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

/** FPS / CPS / keystrokes overlay. Toggled from the Right-Shift menu. */
public class HudOverlay {

    private static final int BG = 0x80000000;
    private static final int BG_PRESSED = 0xCCFFFFFF;
    private static final int TEXT = 0xFFFFFFFF;
    private static final int TEXT_DIM = 0xFFAAAAAA;
    private static final int TEXT_DARK = 0xFF000000;

    public static void init() {
        HudRenderCallback.EVENT.register(HudOverlay::render);
    }

    private static void render(DrawContext context, float tickDelta) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.options.hudHidden || client.options.debugEnabled) {
            return;
        }
        if (DontCamFabricMod.config.fps) {
            drawFps(client, context);
        }
        if (DontCamFabricMod.config.cps) {
            drawCps(client, context);
        }
        if (DontCamFabricMod.config.keystrokes) {
            drawKeystrokes(client, context);
        }
    }

    private static void drawFps(MinecraftClient client, DrawContext context) {
        String text = client.getCurrentFps() + " FPS";
        int w = client.textRenderer.getWidth(text);
        context.fill(5, 5, 9 + w, 18, BG);
        context.drawText(client.textRenderer, text, 7, 7, TEXT, false);
    }

    private static void drawCps(MinecraftClient client, DrawContext context) {
        ClickTracker tracker = ClickTracker.get();
        int left = tracker != null ? tracker.leftCps() : 0;
        int right = tracker != null ? tracker.rightCps() : 0;
        String text = left + " | " + right + " CPS";
        int w = client.textRenderer.getWidth(text);
        context.fill(5, 21, 9 + w, 34, BG);
        context.drawText(client.textRenderer, text, 7, 23, TEXT, false);
    }

    private static void key(MinecraftClient client, DrawContext context, int x, int y, int w, int h, String label, boolean pressed) {
        context.fill(x, y, x + w, y + h, pressed ? BG_PRESSED : BG);
        int color = pressed ? TEXT_DARK : TEXT_DIM;
        int tw = client.textRenderer.getWidth(label);
        context.drawText(client.textRenderer, label, x + (w - tw) / 2, y + (h - 8) / 2, color, false);
    }

    private static void drawKeystrokes(MinecraftClient client, DrawContext context) {
        int key = 22;
        int gap = 2;
        int spaceH = 12;
        int totalW = key * 3 + gap * 2;
        int x0 = context.getScaledWindowWidth() / 2 - totalW / 2;
        int y0 = context.getScaledWindowHeight() - 68;

        boolean w = client.options.forwardKey.isPressed();
        boolean a = client.options.leftKey.isPressed();
        boolean s = client.options.backKey.isPressed();
        boolean d = client.options.rightKey.isPressed();
        boolean space = client.options.jumpKey.isPressed();
        boolean lmb = client.options.attackKey.isPressed();
        boolean rmb = client.options.useKey.isPressed();

        key(client, context, x0 + key + gap, y0, key, key, "W", w);
        key(client, context, x0, y0 + key + gap, key, key, "A", a);
        key(client, context, x0 + key + gap, y0 + key + gap, key, key, "S", s);
        key(client, context, x0 + key * 2 + gap * 2, y0 + key + gap, key, key, "D", d);

        int sy = y0 + key * 2 + gap * 2;
        context.fill(x0, sy, x0 + totalW, sy + spaceH, space ? BG_PRESSED : BG);

        int my = sy + spaceH + gap;
        int half = (totalW - gap) / 2;
        key(client, context, x0, my, half, 12, "LMB", lmb);
        key(client, context, x0 + half + gap, my, totalW - half - gap, 12, "RMB", rmb);
    }
}
