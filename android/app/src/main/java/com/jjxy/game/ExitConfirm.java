package com.jjxy.game;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.DialogInterface;
import android.content.res.Configuration;

/**
 * STAGE-B-005（系统对话框版）：游戏内右滑退出确认——Android 标准 {@link AlertDialog}。
 *
 * <p>用户 2026-10-27："不用游戏内UI，直接用Android标准关闭UI，作为shell层实现，提醒效果更明确"。
 * 原生侧（shell/android/src/exit_dialog.rs，JNI 工作线程）调 {@link #show}，未决期间轮询 {@link #take}；
 * 游戏核心零改动。主题跟随系统深色/浅色（DeviceDefault）。
 */
public final class ExitConfirm {
    /** 无结果（未决或已取走）。 */
    public static final int NONE = 0;
    /** 选"退出"。 */
    public static final int EXIT = 1;
    /** 取消（"取消"按钮 / 返回 / 点框外 / 无法弹出）。 */
    public static final int CANCEL = 2;

    private static final Object LOCK = new Object();
    private static int result = NONE;

    private ExitConfirm() {}

    /** 任意线程调用：清旧结果，在 UI 线程弹出确认框。 */
    public static void show(final Activity activity) {
        post(NONE);
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (activity.isFinishing()) {
                    post(CANCEL);
                    return;
                }
                boolean night = (activity.getResources().getConfiguration().uiMode
                        & Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES;
                int theme = night
                        ? android.R.style.Theme_DeviceDefault_Dialog_Alert
                        : android.R.style.Theme_DeviceDefault_Light_Dialog_Alert;
                new AlertDialog.Builder(activity, theme)
                        .setTitle("退出游戏")
                        .setMessage("是否确定退出游戏？未保存的进度将会丢失。")
                        .setNegativeButton("取消", new DialogInterface.OnClickListener() {
                            @Override
                            public void onClick(DialogInterface dialog, int which) {
                                post(CANCEL);
                            }
                        })
                        .setPositiveButton("退出", new DialogInterface.OnClickListener() {
                            @Override
                            public void onClick(DialogInterface dialog, int which) {
                                post(EXIT);
                            }
                        })
                        .setOnCancelListener(new DialogInterface.OnCancelListener() {
                            @Override
                            public void onCancel(DialogInterface dialog) {
                                post(CANCEL);
                            }
                        })
                        .show();
            }
        });
    }

    /** 原生轮询：取走结果（取后清零）。 */
    public static int take() {
        synchronized (LOCK) {
            int r = result;
            result = NONE;
            return r;
        }
    }

    private static void post(int r) {
        synchronized (LOCK) {
            result = r;
        }
    }
}
