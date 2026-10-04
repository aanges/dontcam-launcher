package com.dontcam.fabric.gui;

import com.dontcam.fabric.ClientConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Square toggle tile for one HUD module. Left-click toggles, right-click opens settings. */
public class DontCamTileWidget extends ButtonWidget {

    private static final int BG = 0xCC101018;
    private static final int BG_HOVER = 0xFF1B63D6;
    private static final int ON = 0xFF3DFF7A;
    private static final int OFF = 0xFFFF5555;

    private final ClientConfig.Module module;
    private final String name;

    public DontCamTileWidget(int x, int y, int size, String name, ClientConfig.Module module, PressAction onPress) {
        super(x, y, size, size, Text.empty(), onPress, DEFAULT_NARRATION_SUPPLIER);
        this.module = module;
        this.name = name;
    }

    @Override
    public void renderWidget(DrawContext context, int mouseX, int mouseY, float delta) {
        boolean hovered = this.isHovered();
        int bg = hovered ? BG_HOVER : BG;
        int edge = module.enabled ? ON : OFF;
        int x = this.getX();
        int y = this.getY();
        int s = this.width;
        context.fill(x, y, x + s, y + s, bg);
        // colored state border
        context.fill(x, y, x + s, y + 2, edge);
        context.fill(x, y + s - 2, x + s, y + s, edge);
        context.fill(x, y, x + 2, y + s, edge);
        context.fill(x + s - 2, y, x + s, y + s, edge);
        if (hovered) {
            context.fill(x + 2, y + 2, x + s - 2, y + 5, 0x33FFFFFF);
        }
        TextRendererHolder.drawCentered(context, name, x + s / 2, y + s / 2 - 8, 0xFFFFFFFF);
        TextRendererHolder.drawCentered(context, module.enabled ? "ON" : "OFF",
                x + s / 2, y + s / 2 + 4, edge);
    }

    /** Tiny indirection so the tile doesn't need a client instance for text. */
    static final class TextRendererHolder {
        static void drawCentered(DrawContext context, String text, int x, int y, int color) {
            MinecraftClient client = MinecraftClient.getInstance();
            int tw = client.textRenderer.getWidth(text);
            context.drawText(client.textRenderer, text, x - tw / 2, y, color, false);
        }
    }
}
