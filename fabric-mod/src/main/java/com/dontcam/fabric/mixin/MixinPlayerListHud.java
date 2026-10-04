package com.dontcam.fabric.mixin;

import com.dontcam.fabric.DontCamBadges;
import com.dontcam.fabric.DontCamFabricMod;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.PlayerListHud;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.scoreboard.ScoreboardObjective;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;
import java.util.UUID;

/**
 * Tab-list badges: tag.png icon + "DC" before the nick, gold star for the owner.
 *
 * <p>Text comes from {@code getPlayerName} (vanilla pipeline); the icon is overlaid
 * in {@code render} TAIL with best-effort column layout. All icon work is wrapped
 * so a layout mismatch can never crash the tab list.
 */
@Mixin(PlayerListHud.class)
public abstract class MixinPlayerListHud {

    @Inject(method = "getPlayerName", at = @At("RETURN"), cancellable = true)
    private void dontcam$badge(PlayerListEntry entry, CallbackInfoReturnable<Text> cir) {
        if (entry == null || entry.getProfile() == null) {
            return;
        }
        UUID uuid = entry.getProfile().getId();
        if (!DontCamFabricMod.isDontCam(uuid)) {
            return;
        }
        cir.setReturnValue(DontCamBadges.badgeLine(cir.getReturnValue(), DontCamFabricMod.isOwner(uuid)));
    }

    @Inject(method = "render", at = @At("TAIL"))
    private void dontcam$tagIcons(DrawContext context, int scaledWindowWidth,
                                  Scoreboard scoreboard, ScoreboardObjective objective, CallbackInfo ci) {
        try {
            MinecraftClient client = MinecraftClient.getInstance();
            if (client.player == null || client.player.networkHandler == null) {
                return;
            }
            List<PlayerListEntry> entries = client.player.networkHandler.getPlayerList().stream()
                    .limit(80)
                    .toList();
            if (entries.isEmpty()) {
                return;
            }
            // Vanilla lays the tab list out in columns of up to 20 rows. Mirror that
            // loosely: column width 300px capped, rows of 20.
            int cols = (entries.size() + 19) / 20;
            cols = Math.max(1, Math.min(4, cols));
            int colWidth = Math.min(300, scaledWindowWidth / Math.max(1, cols) - 40);
            int totalW = cols * colWidth;
            int x0 = scaledWindowWidth / 2 - totalW / 2;
            int y0 = 20;
            for (int idx = 0; idx < entries.size(); idx++) {
                PlayerListEntry entry = entries.get(idx);
                if (entry == null || entry.getProfile() == null) {
                    continue;
                }
                UUID uuid = entry.getProfile().getId();
                if (!DontCamFabricMod.isDontCam(uuid)) {
                    continue;
                }
                int col = idx / 20;
                int row = idx % 20;
                int x = x0 + col * colWidth + 4;
                int y = y0 + row * 9;
                DontCamBadges.drawTagIcon(context, x, y - 1, 8);
            }
        } catch (Exception ignored) {
        }
    }
}
