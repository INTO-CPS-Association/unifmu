#!/usr/bin/env python
"""
Hello World, but with more meat.
"""

from os import spawnl
from sys import flags
import wx


class CreateFMUFrame(wx.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent, title="Create new FMU")

        panel = wx.Panel(self)

        # controls

        name_label = wx.StaticText(panel, label="Name:")
        self.name_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.name_field.SetToolTip("Human-readable identifier assoicated with the FMU")

        author_label = wx.StaticText(panel, label="Author:")
        self.author_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.author_field.SetToolTip("The author of the generated FMU")

        description_label = wx.StaticText(panel, label="Description:")
        self.description_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.description_field.SetToolTip("A guide describing how to use the FMU")

        self.fmi_selector = wx.RadioBox(
            panel,
            label="FMI version",
            choices=["1.0", "2.0", "3.0"],
            majorDimension=1,
            style=wx.RA_SPECIFY_ROWS,
        )
        self.fmi_selector.SetToolTip(
            "Which version of the Functional Mock-Up interface the FMU targets"
        )

        backend_label = wx.StaticText(panel, label="Backend")
        self.backend_combo = wx.ComboBox(
            panel, value="None", choices=["None", "UniFMU"], style=wx.CB_READONLY
        )
        self.backend_combo.SetToolTip(
            "Copy an ready-to use backend into the generated FMU, providing a quick way to get started"
        )

        self.button_generate = wx.Button(panel, label="Generate")
        self.button_cancel = wx.Button(panel, label="Cancel")

        self.Bind(wx.EVT_BUTTON, self.on_generate, self.button_generate)
        self.Bind(wx.EVT_BUTTON, self.on_cancel, self.button_cancel)

        # sizers
        border_size = 5

        name_sizer = wx.BoxSizer()
        name_sizer.Add(name_label, 0, wx.ALL, border_size)
        name_sizer.Add(self.name_field, 1, wx.ALL, border_size)

        author_sizer = wx.BoxSizer()
        author_sizer.Add(author_label, 0, wx.ALL, border_size)
        author_sizer.Add(self.author_field, 1, wx.ALL, border_size)

        description_sizer = wx.BoxSizer()
        description_sizer.Add(description_label, 0, wx.ALL, border_size)
        description_sizer.Add(self.description_field, 1, wx.ALL, border_size)

        border_size = 5
        fmi_sizer = wx.BoxSizer()
        fmi_sizer.Add(self.fmi_selector, 1, wx.ALL, border_size)
        fmi_sizer.Add(backend_label, 0, wx.ALL, border_size)
        fmi_sizer.Add(self.backend_combo, 1, wx.ALL, border_size)

        button_sizer = wx.BoxSizer()
        button_sizer.Add(self.button_cancel, 1, wx.ALL, border_size)
        button_sizer.Add(self.button_generate, 1, wx.ALL, border_size)

        main_sizer = wx.BoxSizer(orient=wx.VERTICAL)
        main_sizer.Add(name_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(author_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(description_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(fmi_sizer, 0, wx.ALL | wx.EXPAND | wx.CENTER, border_size)
        main_sizer.Add(button_sizer, 0, wx.ALL | wx.CENTER, border_size)

        # fit controls
        panel.SetSizerAndFit(main_sizer)
        self.Fit()
        # self.Show()

    def on_generate(self, event):
        print("generating")

    def on_cancel(self, event):
        self.Close()


class HomeScreenFrame(wx.Frame):
    """
    A Frame that says Hello World
    """

    def __init__(self, title: str):
        # ensure the parent's __init__ is called
        super().__init__(None, title="FMU Builder")

        # create a panel in the frame
        pnl = wx.Panel(self)

        # put some text with a larger bold font on it
        st = wx.StaticText(pnl, label="Hello World!")
        font = st.GetFont()
        font.PointSize += 10
        font = font.Bold()
        st.SetFont(font)

        # and create a sizer to manage the layout of child widgets
        sizer = wx.BoxSizer(wx.VERTICAL)
        sizer.Add(st, wx.SizerFlags().Border(wx.TOP | wx.LEFT, 25))
        pnl.SetSizer(sizer)

        # create a menu bar
        self.makeMenuBar()

        # and a status bar
        self.CreateStatusBar()
        self.SetStatusText("Waiting for user")

    def makeMenuBar(self):
        """
        A menu bar is composed of menus, which are composed of menu items.
        This method builds a set of menus and binds handlers to be called
        when the menu item is selected.
        """

        # Make a file menu with Hello and Exit items
        fileMenu = wx.Menu()
        # The "\t..." syntax defines an accelerator key that also triggers
        # the same event
        createItem = fileMenu.Append(
            -1, "&Create FMU...\tCtrl-H", "Generate a template for a new FMU",
        )
        fileMenu.AppendSeparator()

        EditItem = fileMenu.Append(-1, "&Edit FMU...\tCtrl-H", "Modify existing FMU",)

        fileMenu.AppendSeparator()

        # When using a stock ID we don't need to specify the menu item's
        # label
        exitItem = fileMenu.Append(wx.ID_EXIT)

        # Now a help menu for the about item
        helpMenu = wx.Menu()
        aboutItem = helpMenu.Append(wx.ID_ABOUT)

        # Make the menu bar and add the two menus to it. The '&' defines
        # that the next letter is the "mnemonic" for the menu item. On the
        # platforms that support it those letters are underlined and can be
        # triggered from the keyboard.
        menuBar = wx.MenuBar()
        menuBar.Append(fileMenu, "&File")
        menuBar.Append(helpMenu, "&Help")

        # Give the menu bar to the frame
        self.SetMenuBar(menuBar)

        # Finally, associate a handler function with the EVT_MENU event for
        # each of the menu items. That means that when that menu item is
        # activated then the associated handler function will be called.
        self.Bind(wx.EVT_MENU, self.on_create, createItem)
        self.Bind(wx.EVT_MENU, self.on_exit, exitItem)
        self.Bind(wx.EVT_MENU, self.on_about, aboutItem)

    def on_exit(self, event):
        """Close the frame, terminating the application."""
        self.Close(True)

    def on_create(self, event):
        """Say hello to the user."""

        # with wx.DirDialog(
        #     self, "Select a directory to place the generated FMU inside"
        # ) as dirDialog:

        #     if dirDialog.ShowModal() != wx.ID_CANCEL:
        #         pathname = dirDialog.GetPath()
        #         print(f"path is {pathname}")

        frame = CreateFMUFrame(None)
        frame.Show()

    def on_edit(self, event):
        wx.DirDialog(self)
        pass

    def on_about(self, event):
        """Display an About Dialog"""
        wx.MessageBox(
            "This is a wxPython Hello World sample",
            "About Hello World 2",
            wx.OK | wx.ICON_INFORMATION,
        )


if __name__ == "__main__":
    # When this module is run (not imported) then create the app, the
    # frame, show it, and start the event loop.
    app = wx.App()
    frm = HomeScreenFrame(title="FMU Builder")
    frm.Show()
    app.MainLoop()
