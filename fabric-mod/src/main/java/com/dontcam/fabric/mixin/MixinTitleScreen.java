package com.dontcam.fabric.mixin;

import com.dontcam.fabric.DontCamBadges;
import com.dontcam.fabric.DontCamFabricMod;
import com.dontcam.fabric.gui.ClientOptionsScreen;
import com.dontcam.fabric.gui.DontCamButtonWidget;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Reworked main menu: two columns — MINECRAFT (vanilla entries) next to
 * DONTCAM (client options, tag status, auth) — under a shared banner.
 */
@Mixin(TitleScreen.class)
public abstract class MixinTitleScreen extends Screen {

    @Unique
    private int dontcam$leftX;
    @Unique
    private int dontcam$rightX;
    @Unique
    private int dontcam$colY;

    protected MixinTitleScreen(Text title) {
        super(title);
    }

    @Unique
    private void dontcam$addButton(int x, int y, String label, Runnable action) {
        this.addDrawableChild(new DontCamButtonWidget(x, y, 180, 20,
                Text.literal(label), button -> action.run()));
    }

    @Inject(method = "init", at = @At("TAIL"))
    private void dontcam$replaceButtons(CallbackInfo ci) {
        this.clearChildren();
        dontcam$colY = this.height / 2 - 8;
        dontcam$leftX = this.width / 2 - 190;
        dontcam$rightX = this.width / 2 + 10;

        dontcam$addButton(dontcam$leftX, dontcam$colY, "Singleplayer", () -> {
            if (this.client != null) {
                this.client.setScreen(new SelectWorldScreen(this));
            }
        });
        dontcam$addButton(dontcam$leftX, dontcam$colY + 24, "Multiplayer", () -> {
            if (this.client != null) {
                this.client.setScreen(new MultiplayerScreen(this));
            }
        });
        dontcam$addButton(dontcam$leftX, dontcam$colY + 48, "Options", () -> {
            if (this.client != null && this.client.options != null) {
                this.client.setScreen(new OptionsScreen(this, this.client.options));
            }
        });
        dontcam$addButton(dontcam$leftX, dontcam$colY + 72, "Quit Game", () -> {
            if (this.client != null) {
                this.client.scheduleStop();
            }
        });

        dontcam$addButton(dontcam$rightX, dontcam$colY, "Client Options", () -> {
            if (this.client != null) {
                this.client.setScreen(new ClientOptionsScreen());
            }
        });
        dontcam$addButton(dontcam$rightX, dontcam$colY + 24, "Key Binds", () -> {
            dontcam$openVanillaOptions();
        });
        dontcam$addButton(dontcam$rightX, dontcam$colY + 48, "Resource Packs", () -> {
            dontcam$openVanillaOptions();
        });
        dontcam$addButton(dontcam$rightX, dontcam$colY + 72, "Language", () -> {
            dontcam$openVanillaOptions();
        });
    }

    @Unique
    private void dontcam$openVanillaOptions() {
        if (this.client != null && this.client.options != null) {
            this.client.setScreen(new OptionsScreen(this, this.client.options));
        }
    }

    @Inject(method = "render", at = @At("TAIL"))
    private void dontcam$banner(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        int cx = this.width / 2;
        // Banner covering the vanilla logo + splash zone.
        context.fill(cx - 230, 18, cx + 230, 92, 0xCC0A0A12);
        context.fill(cx - 230, 18, cx + 230, 19, 0xFF2E9BFF);
        context.fill(cx - 230, 91, cx + 230, 92, 0xFF2E9BFF);
        // tag.png logo left of the title.
        DontCamBadges.drawTagIcon(context, cx - 118, 30, 24);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal("DontCam Client"), cx + 12, 32, 0xFFFFFFFF);
        String auth = DontCamFabricMod.localMicrosoft ? "\u00A7aMicrosoft" : "\u00A77Offline";
        String line = "DC tag always on  \u2022  " + auth + "  \u2022  v0.1.0";
        int tw = this.textRenderer.getWidth(line);
        context.drawText(this.textRenderer, line, cx + 12 - tw / 2, 50, 0xFF7DD3FC, false);
        String ver = "Minecraft " + (this.client != null && this.client.getGameVersion() != null
                ? this.client.getGameVersion() : "1.21.1");
        int vw = this.textRenderer.getWidth(ver);
        context.drawText(this.textRenderer, ver, cx + 12 - vw / 2, 64, 0xFF64748B, false);
        // Column headers above each button column.
        context.drawCenteredTextWithShadow(this.textRenderer, Text.literal("MINECRAFT"),
                dontcam$leftX + 90, dontcam$colY - 14, 0xFF7DD3FC);
        context.drawCenteredTextWithShadow(this.textRenderer, Text.literal("DONTCAM"),
                dontcam$rightX + 90, dontcam$colY - 14, 0xFF7DD3FC);
    }
}
