package com.dontcam.fabric;

import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormats;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import net.minecraft.util.Identifier;
import org.joml.Matrix4f;

/**
 * Shared DC badge helpers + {@code tag.png} icon rendering.
 *
 * <p>Texture lives at {@code assets/dontcam/textures/tag.png} and is referenced
 * as {@code dontcam:textures/tag.png}. All badge/icon rendering is gated on
 * {@link DontCamFabricMod#config}{@code .badges} by the callers.
 */
public final class DontCamBadges {

    /** Small infinity-logo icon drawn before the nickname. */
    public static final Identifier TAG_TEXTURE = Identifier.of("dontcam", "textures/tag.png");

    private DontCamBadges() {
    }

    /** {@code D}(blue,bold) + {@code C}(aqua,bold) + space + name [+ gold star for the owner]. */
    public static MutableText badgeLine(Text name, boolean owner) {
        MutableText line = Text.literal("D").formatted(Formatting.BLUE, Formatting.BOLD)
                .append(Text.literal("C").formatted(Formatting.AQUA, Formatting.BOLD))
                .append(Text.literal(" "))
                .append(name.copy());
        if (owner) {
            line.append(Text.literal(" \u2605").formatted(Formatting.GOLD, Formatting.BOLD));
        }
        return line;
    }

    /** 2D icon for screens / HUD / tab-list overlays. */
    public static void drawTagIcon(DrawContext context, int x, int y, int size) {
        try {
            context.drawTexture(TAG_TEXTURE, x, y, 0, 0, size, size, size, size);
        } catch (Exception ignored) {
        }
    }

    /**
     * World-space icon for the overhead nametag. Draws a {@code size x size} textured
     * quad on the current label matrix at {@code (x, y)}, then the caller draws text.
     */
    public static void drawTagIconWorld(MatrixStack matrices, float x, float y, float size) {
        try {
            MinecraftClient client = MinecraftClient.getInstance();
            client.getTextureManager().bindTexture(TAG_TEXTURE);
            RenderSystem.setShader(GameRenderer::getPositionTexProgram);
            RenderSystem.setShaderTexture(0, TAG_TEXTURE);
            Matrix4f matrix = matrices.peek().getPositionMatrix();
            Tessellator tessellator = Tessellator.getInstance();
            BufferBuilder buffer = tessellator.begin(VertexFormat.DrawMode.QUADS, VertexFormats.POSITION_TEXTURE);
            buffer.vertex(matrix, x, y + size, 0.0f).texture(0.0f, 1.0f);
            buffer.vertex(matrix, x + size, y + size, 0.0f).texture(1.0f, 1.0f);
            buffer.vertex(matrix, x + size, y, 0.0f).texture(1.0f, 0.0f);
            buffer.vertex(matrix, x, y, 0.0f).texture(0.0f, 0.0f);
            BufferRenderer.drawWithGlobalProgram(buffer.end());
        } catch (Exception ignored) {
        }
    }
}
