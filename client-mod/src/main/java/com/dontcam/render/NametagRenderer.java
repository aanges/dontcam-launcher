package com.dontcam.render;

import com.dontcam.DontCamMod;
import com.dontcam.DontCamMod;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraftforge.client.event.RenderPlayerEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import org.lwjgl.opengl.GL11;

import java.util.UUID;

/**
 * Replaces the vanilla nametag with a DontCam one for known players:
 * a merged "DC" mark in front of the nick, plus a gold crown for the owner
 * (owner crown only counts on Microsoft sessions).
 */
public class NametagRenderer {

    private static final int gatheringBlue = 0xFF5555FF;
    private static final int gatheringCyan = 0xFF55FFFF;
    private static final int gold = 0xFFFFAA00;

    @SubscribeEvent
    public void onRenderSpecials(RenderPlayerEvent.Specials.Pre event) {
        if (!DontCamMod.config.badges) {
            return;
        }
        EntityPlayer player = event.entityPlayer;
        if (player.isSneaking() || player.isInvisible()) {
            return;
        }
        UUID uuid = player.getUniqueID();
        if (!DontCamMod.isDontCam(uuid)) {
            return;
        }
        event.setCanceled(true);

        boolean owner = DontCamMod.isOwner(uuid);
        Minecraft mc = Minecraft.getMinecraft();
        FontRenderer font = mc.fontRendererObj;
        RenderManager rm = mc.getRenderManager();

        String name = player.getDisplayName().getFormattedText();
        int nameW = font.getStringWidth(name);
        int badgeW = 12;
        int total = nameW + badgeW;

        float scale = 0.016666668F * 1.6F;
        GlStateManager.pushMatrix();
        GlStateManager.translate((float) event.x, (float) event.y + player.height + 0.5F, (float) event.z);
        GL11.glNormal3f(0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(rm.playerViewX, 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(-scale, -scale, scale);
        GlStateManager.disableLighting();
        GlStateManager.depthMask(false);
        GlStateManager.disableDepth();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);

        // Backplate
        Gui.drawRect(-total / 2 - 1, -1, total / 2 + 1, 9, 0x80000000);

        int cx = -total / 2 + 1;
        // Merged "DC" mark: cyan C with a blue D overlapping it by 1px.
        font.drawString("C", cx, 0, gatheringCyan);
        font.drawString("D", cx - 1, 0, gatheringBlue);
        font.drawString(name, cx + badgeW, 0, 0xFFFFFFFF);

        if (owner) {
            drawCrown(-5, -8);
        }

        GlStateManager.depthMask(true);
        GlStateManager.enableDepth();
        GlStateManager.disableBlend();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.popMatrix();
    }

    /** Tiny gold crown centered at (x, y). */
    private void drawCrown(int x, int y) {
        // base bar
        Gui.drawRect(x, y + 3, x + 10, y + 5, gold);
        // spikes
        Gui.drawRect(x, y, x + 2, y + 3, gold);
        Gui.drawRect(x + 4, y - 1, x + 6, y + 3, gold);
        Gui.drawRect(x + 8, y, x + 10, y + 3, gold);
    }
}
