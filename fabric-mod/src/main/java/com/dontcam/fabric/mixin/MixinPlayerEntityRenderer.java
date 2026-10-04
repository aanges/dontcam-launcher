package com.dontcam.fabric.mixin;

import com.dontcam.fabric.DontCamBadges;
import com.dontcam.fabric.DontCamFabricMod;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.render.entity.PlayerEntityRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.UUID;

/**
 * Custom nametag for DontCam players: "DC" badge before the nick,
 * gold star for the owner (Microsoft sessions only).
 */
@Mixin(PlayerEntityRenderer.class)
public abstract class MixinPlayerEntityRenderer {

    @Inject(method = "renderLabelIfPresent", at = @At("HEAD"), cancellable = true)
    private void dontcam$label(AbstractClientPlayerEntity player, Text text, MatrixStack matrices,
                               VertexConsumerProvider vertexConsumers, int light, float tickDelta,
                               CallbackInfo ci) {
        UUID uuid = player.getUuid();
        if (!DontCamFabricMod.isDontCam(uuid)) {
            return;
        }
        ci.cancel();

        boolean owner = DontCamFabricMod.isOwner(uuid);
        MutableText line = DontCamBadges.badgeLine(text, owner);

        MinecraftClient client = MinecraftClient.getInstance();
        TextRenderer textRenderer = client.textRenderer;
        float icon = 8.0f;
        float gap = 2.0f;
        float width = -(textRenderer.getWidth(line) + icon + gap) / 2.0f;

        matrices.push();
        matrices.translate(0.0, player.getHeight() + 0.5, 0.0);
        matrices.multiply(client.getEntityRenderDispatcher().getRotation());
        matrices.scale(-0.025f, -0.025f, 0.025f);
        // Icon first, then text shifted right so they sit side by side.
        DontCamBadges.drawTagIconWorld(matrices, width, -1.0f, icon);
        float textX = width + icon + gap;
        textRenderer.draw(line, textX, 0.0f, 0xFFFFFFFF, false,
                matrices.peek().getPositionMatrix(), vertexConsumers,
                TextRenderer.TextLayerType.NORMAL, 0x80000000, light);
        matrices.pop();
    }
}
