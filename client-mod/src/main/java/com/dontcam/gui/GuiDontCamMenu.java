package com.dontcam.gui;

import com.dontcam.DontCamMod;
import com.dontcam.DontCamMod;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiScreen;

import java.io.IOException;

/** Right-Shift menu: toggle the DontCam modules. */
public class GuiDontCamMenu extends GuiScreen {

    private static final int ID_FPS = 0;
    private static final int ID_CPS = 1;
    private static final int ID_KEYS = 2;
    private static final int ID_BADGES = 3;
    private static final int ID_DONE = 99;

    @Override
    public void initGui() {
        int cx = this.width / 2;
        int y = this.height / 2 - 44;
        this.buttonList.add(new GuiButton(ID_FPS, cx - 100, y, 200, 20, ""));
        this.buttonList.add(new GuiButton(ID_CPS, cx - 100, y + 24, 200, 20, ""));
        this.buttonList.add(new GuiButton(ID_KEYS, cx - 100, y + 48, 200, 20, ""));
        this.buttonList.add(new GuiButton(ID_BADGES, cx - 100, y + 72, 200, 20, ""));
        this.buttonList.add(new GuiButton(ID_DONE, cx - 100, y + 100, 200, 20, "Done"));
        refreshLabels();
    }

    private void refreshLabels() {
        for (GuiButton button : this.buttonList) {
            switch (button.id) {
                case ID_FPS:
                    button.displayString = "FPS Counter: " + onOff(DontCamMod.config.fps);
                    break;
                case ID_CPS:
                    button.displayString = "CPS Counter: " + onOff(DontCamMod.config.cps);
                    break;
                case ID_KEYS:
                    button.displayString = "Keystrokes: " + onOff(DontCamMod.config.keystrokes);
                    break;
                case ID_BADGES:
                    button.displayString = "DC Badges: " + onOff(DontCamMod.config.badges);
                    break;
                default:
                    break;
            }
        }
    }

    private static String onOff(boolean value) {
        return value ? "§aON" : "§cOFF";
    }

    @Override
    protected void actionPerformed(GuiButton button) throws IOException {
        switch (button.id) {
            case ID_FPS:
                DontCamMod.config.fps = !DontCamMod.config.fps;
                break;
            case ID_CPS:
                DontCamMod.config.cps = !DontCamMod.config.cps;
                break;
            case ID_KEYS:
                DontCamMod.config.keystrokes = !DontCamMod.config.keystrokes;
                break;
            case ID_BADGES:
                DontCamMod.config.badges = !DontCamMod.config.badges;
                break;
            case ID_DONE:
                DontCamMod.config.save();
                this.mc.displayGuiScreen(null);
                return;
            default:
                return;
        }
        DontCamMod.config.save();
        refreshLabels();
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        this.drawDefaultBackground();
        this.drawCenteredString(this.fontRendererObj, "§bDontCam §9Client", this.width / 2, this.height / 2 - 70, 0xFFFFFF);
        String auth = DontCamMod.localMicrosoft ? "§aMicrosoft" : "§7Offline";
        this.drawCenteredString(this.fontRendererObj, auth, this.width / 2, this.height / 2 - 58, 0xFFFFFF);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
