package com.dontcam.fabric.mixin;

import com.dontcam.fabric.DontCamFabricMod;
import net.minecraft.client.gui.hud.PlayerListHud;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.UUID;

/** Tab-list badges: "DC" before the nick, gold star for the owner. */
@Mixin(PlayerListHud.class)
public abstract class MixinPlayerListHud {

    @Inject(method = "getPlayerName", at = @At("RETURN"), cancellable = true)
    private void dontcam$badge(PlayerListEntry entry, CallbackInfoReturnable<Text> cir) {
        if (!DontCamFabricMod.config.badges || entry == null || entry.getProfile() == null) {
            return;
        }
        UUID uuid = entry.getProfile().getId();
        if (!DontCamFabricMod.isDontCam(uuid)) {
            return;
        }
        MutableText line = Text.literal("D").formatted(Formatting.BLUE, Formatting.BOLD)
                .append(Text.literal("C").formatted(Formatting.AQUA, Formatting.BOLD))
                .append(Text.literal(" "))
                .append(cir.getReturnValue().copy());
        if (DontCamFabricMod.isOwner(uuid)) {
            line.append(Text.literal(" ★").formatted(Formatting.GOLD, Formatting.BOLD));
        }
        cir.setReturnValue(line);
    }
}
