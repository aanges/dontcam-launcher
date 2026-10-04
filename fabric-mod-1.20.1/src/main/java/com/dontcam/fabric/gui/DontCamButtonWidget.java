package com.dontcam.fabric.gui;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Rounded button with hover glow, used by DontCam screens. */
public class DontCamButtonWidget extends ButtonWidget {

    private static final int BG = 0xCC101018;
    private static final int BG_HOVER = 0xFF1B63D6;
    private static final int BORDER = 0xFF2E9BFF;
    private static final int BORDER_DIM = 0x552E9BFF;

    public DontCamButtonWidget(int x, int y, int width, int height, Text message, PressAction onPress) {
        super(x, y, width, height, message, onPress, DEFAULT_NARRATION_SUPPLIER);
    }

    @Override
    public void renderButton(DrawContext context, int mouseX, int mouseY, float delta) {
        boolean hovered = this.isHovered();
        int bg = hovered ? BG_HOVER : BG;
        int border = hovered ? BORDER : BORDER_DIM;
        int x = this.getX();
        int y = this.getY();
        int w = this.width;
        int h = this.height;
        // body
        context.fill(x + 1, y, x + w - 1, y + h, bg);
        context.fill(x, y + 1, x + w, y + h - 1, bg);
        // border
        context.fill(x + 1, y, x + w - 1, y + 1, border);
        context.fill(x + 1, y + h - 1, x + w, y + h, border);
        context.fill(x, y + 1, x + 1, y + h - 1, border);
        context.fill(x + w - 1, y + 1, x + w, y + h - 1, border);
        if (hovered) {
            context.fill(x + 1, y + 1, x + w - 1, y + 3, 0x33FFFFFF);
        }
        int color = this.active ? 0xFFFFFFFF : 0xFFA0A0A0;
        context.drawCenteredTextWithShadow(
                net.minecraft.client.MinecraftClient.getInstance().textRenderer,
                this.getMessage(), x + w / 2, y + (h - 8) / 2, color);
    }
}
