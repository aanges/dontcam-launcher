package com.dontcam.fabric.mixin;

import com.dontcam.fabric.gui.DontCamButtonWidget;
import com.dontcam.fabric.gui.DontCamMenuScreen;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Replaces the vanilla main menu buttons with pretty DontCam ones and paints
 * a minimal "DontCam Client" banner over the logo area.
 */
@Mixin(TitleScreen.class)
public abstract class MixinTitleScreen extends Screen {

    protected MixinTitleScreen(Text title) {
        super(title);
    }

    @Inject(method = "init", at = @At("TAIL"))
    private void dontcam$replaceButtons(CallbackInfo ci) {
        this.clearChildren();
        int cx = this.width / 2;
        int y = this.height / 2 - 20;
        this.addDrawableChild(new DontCamButtonWidget(cx - 100, y, 200, 20,
                Text.literal("Singleplayer"),
                button -> {
                    if (this.client != null) {
                        this.client.setScreen(new SelectWorldScreen(this));
                    }
                }));
        this.addDrawableChild(new DontCamButtonWidget(cx - 100, y + 24, 200, 20,
                Text.literal("Multiplayer"),
                button -> {
                    if (this.client != null) {
                        this.client.setScreen(new MultiplayerScreen(this));
                    }
                }));
        this.addDrawableChild(new DontCamButtonWidget(cx - 100, y + 48, 200, 20,
                Text.literal("DontCam Mods"),
                button -> {
                    if (this.client != null) {
                        this.client.setScreen(new DontCamMenuScreen());
                    }
                }));
        this.addDrawableChild(new DontCamButtonWidget(cx - 100, y + 72, 200, 20,
                Text.literal("Quit Game"),
                button -> {
                    if (this.client != null) {
                        this.client.scheduleStop();
                    }
                }));
    }

    @Inject(method = "render", at = @At("TAIL"))
    private void dontcam$banner(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        int cx = this.width / 2;
        // Banner covering the vanilla logo + splash zone.
        context.fill(cx - 230, 18, cx + 230, 92, 0xCC0A0A12);
        context.fill(cx - 230, 18, cx + 230, 19, 0xFF2E9BFF);
        context.fill(cx - 230, 91, cx + 230, 92, 0xFF2E9BFF);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal("DontCam Client"), cx, 34, 0xFFFFFFFF);
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal("minimal minecraft client").styled(s -> s.withColor(0xFF7DD3FC)), cx, 52, 0xFFFFFFFF);
        String ver = "v0.1.0";
        if (this.client != null && this.client.getGameVersion() != null) {
            ver += "  •  " + this.client.getGameVersion();
        }
        context.drawCenteredTextWithShadow(this.textRenderer,
                Text.literal(ver).styled(s -> s.withColor(0xFF64748B)), cx, 68, 0xFFFFFFFF);
    }
}
