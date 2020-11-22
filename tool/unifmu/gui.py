from pathlib import Path
import random
from typing import Any
import pubsub

import wx
import wx.adv
from wx.core import BoxSizer
import wx.lib.agw.aui as aui
import wx.gizmos
from pubsub import pub


from unifmu.fmi2 import ModelDescription, CoSimulation
from unifmu.generate import (
    generate_fmu_from_backend,
    get_backends,
    import_fmu,
    export_model_description,
)


class CreateFMUFrame(wx.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent, title="Create new FMU")

        panel = wx.Panel(self)

        # controls

        # name_label = wx.StaticText(panel, label="Name:")
        # self.name_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        # self.name_field.SetToolTip(
        #     "Human-readable identifier assoicated with the FMU")

        # author_label = wx.StaticText(panel, label="Author:")
        # self.author_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        # self.author_field.SetToolTip("The author of the generated FMU")

        # description_label = wx.StaticText(panel, label="Description:")
        # self.description_field = wx.TextCtrl(
        #     panel, style=wx.TE_RICH2 | wx.TE_MULTILINE)
        # self.description_field.SetToolTip(
        #     "A guide describing how to use the FMU")

        # self.fmi_selector = wx.RadioBox(
        #     panel,
        #     label="FMI version",
        #     choices=["1.0", "2.0", "3.0"],
        #     majorDimension=1,
        #     style=wx.RA_SPECIFY_ROWS,
        # )
        # self.fmi_selector.SetSelection(1)
        # self.fmi_selector.SetToolTip(
        #     "Which version of the Functional Mock-Up interface the FMU targets"
        # )

        backend_label = wx.StaticText(panel, label="Backend")
        self.backend_combo = wx.ComboBox(
            panel,
            value=get_backends()[0],
            choices=get_backends(),
            style=wx.CB_READONLY,
        )
        self.backend_combo.SetToolTip(
            "Copy an ready-to use backend into the generated FMU, providing a quick way to get started using a fully functional FMU"
        )

        self.button_generate = wx.Button(panel, label="Generate")
        self.button_cancel = wx.Button(panel, label="Cancel")

        self.Bind(wx.EVT_BUTTON, self.on_generate, self.button_generate)
        self.Bind(wx.EVT_BUTTON, self.on_cancel, self.button_cancel)

        # sizers
        border_size = 5

        # name_sizer = wx.BoxSizer()
        # name_sizer.Add(name_label, 0, wx.ALL, border_size)
        # name_sizer.Add(self.name_field, 1, wx.ALL, border_size)

        # author_sizer = wx.BoxSizer()
        # author_sizer.Add(author_label, 0, wx.ALL, border_size)
        # author_sizer.Add(self.author_field, 1, wx.ALL, border_size)

        # description_sizer = wx.GridBagSizer()
        # description_sizer.Add(
        #    description_label, (0, 0), (1, 1), wx.ALL | wx.EXPAND, border_size
        # )
        # description_sizer.Add(
        #    self.description_field, (0, 1), (1,
        #                                     3), wx.ALL | wx.EXPAND, border_size,
        # )
        # description_sizer.AddGrowableRow(0)
        # description_sizer.AddGrowableCol(1)

        border_size = 5
        fmi_sizer = wx.BoxSizer()
        # fmi_sizer.Add(self.fmi_selector, 1, wx.ALL, border_size)
        fmi_sizer.Add(backend_label, 0, wx.ALL, border_size)
        fmi_sizer.Add(self.backend_combo, 1, wx.ALL, border_size)

        # ------------- export ------------------

        self.outdir_picker = wx.DirPickerCtrl(
            panel, message="Select Base Directory", path=Path.cwd().__fspath__(),
        )
        self.outdir_picker.SetToolTip(
            "Select the directory in which the generated FMU will be written. Note that it will NOT be overwritten; a new directory or zip-archive is created."
        )

        self.filename_field = wx.TextCtrl(panel, style=wx.TE_RICH2, value="MyFMU")

        self.export_zipped_box = wx.CheckBox(panel, label="zip and append .fmu suffix")

        filename_sizer = wx.BoxSizer()
        filename_sizer.AddMany(
            [
                (self.filename_field, 0, wx.ALL | wx.EXPAND, border_size),
                (self.export_zipped_box, 0, wx.ALL | wx.EXPAND, border_size),
            ]
        )

        export_sizer = wx.FlexGridSizer(2)
        export_sizer.AddGrowableCol(1, 1)
        export_sizer.AddMany(
            [
                (wx.StaticText(panel, label="Output Directory:"), 0, wx.ALL),
                (self.outdir_picker, 1, wx.ALL | wx.EXPAND),
                (wx.StaticText(panel, label="Filename:"), 0, wx.ALL),
                (filename_sizer, 1, wx.ALL | wx.EXPAND),
            ]
        )

        # ------------- generate and cancel buttons ----------------
        button_sizer = wx.BoxSizer()
        button_sizer.Add(self.button_cancel, 1, wx.ALL, border_size)
        button_sizer.Add(self.button_generate, 1, wx.ALL, border_size)

        main_sizer = wx.BoxSizer(orient=wx.VERTICAL)
        # main_sizer.Add(name_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        # main_sizer.Add(author_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        # main_sizer.Add(description_sizer, 1, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(fmi_sizer, 0, wx.ALL | wx.EXPAND | wx.CENTER, border_size)
        main_sizer.Add(export_sizer, 1, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(button_sizer, 0, wx.ALL | wx.CENTER, border_size)

        # fit controls
        panel.SetSizerAndFit(main_sizer)
        self.Fit()
        # self.Show()

    def on_generate(self, event):

        output_path = Path(self.outdir_picker.Path) / self.filename_field.Value

        generate_fmu_from_backend(self.backend_combo.Value, output_path)

        # generate_fmu(
        #     mdd, output_path=output_path, backend=self.backend_combo.Value,
        # )
        self.Close()

    def on_cancel(self, event):
        self.Close()


class FMI2BasicPanel(wx.Panel):
    def __init__(self, parent) -> None:

        wx.Panel.__init__(self, parent=parent)

        self.author_field = wx.TextCtrl(self, style=wx.TE_RICH2)
        self.author_field.SetToolTip("Author of the FMU.")
        author_label = wx.StaticText(self, label="Author")

        self.model_identifier_field = wx.TextCtrl(self, style=wx.TE_RICH2)
        self.model_identifier_field.SetToolTip(
            "Name of the shared library (omitting the file name extension), located in the binaries folder of the FMU."
        )
        model_identifier_label = wx.StaticText(self, label="Model Identifier")

        self.name_field = wx.TextCtrl(self, style=wx.TE_RICH2)
        self.name_field.SetToolTip(
            "Name of the FMU typically displayed by the simulation environment."
        )
        name_label = wx.StaticText(self, label="Name")

        description_label = wx.StaticText(self, label="Description")
        self.description_field = wx.TextCtrl(
            self, style=wx.TE_RICH2 | wx.TE_MULTILINE | wx.EXPAND
        )

        sizer = wx.FlexGridSizer(rows=4, cols=2, hgap=5, vgap=5)
        sizer.AddGrowableCol(1)
        sizer.AddGrowableRow(3)
        sizer.Add(name_label)
        sizer.Add(self.name_field, flag=wx.EXPAND)
        sizer.Add(model_identifier_label)
        sizer.Add(self.model_identifier_field, flag=wx.EXPAND)
        sizer.Add(author_label)
        sizer.Add(self.author_field, flag=wx.EXPAND)
        sizer.Add(description_label)
        sizer.Add(self.description_field, flag=wx.EXPAND)

        self.SetSizer(sizer)

        self.model_identifier_field.Bind(
            wx.EVT_TEXT, self.notify_model_identifier_changed
        )
        self.name_field.Bind(wx.EVT_TEXT, self.notify_name_changed)
        self.author_field.Bind(wx.EVT_TEXT, self.notify_author_changed)
        self.description_field.Bind(wx.EVT_TEXT, self.notify_description_changed)

        pub.subscribe(
            self.on_model_identifier_changed, "model.modified.model_identifier"
        )
        pub.subscribe(self.on_name_changed, "model.modified.model_name")
        pub.subscribe(self.on_author_changed, "model.modified.author")
        pub.subscribe(self.on_description_changed, "model.modified.description")

    def notify_model_identifier_changed(self, event):
        pub.sendMessage(
            "model.modified.model_identifier", value=self.model_identifier_field.Value,
        )

    def notify_name_changed(self, event):
        pub.sendMessage("model.modified.model_name", value=self.name_field.Value)

    def notify_author_changed(self, event):
        pub.sendMessage("model.modified.author", value=self.author_field.Value)

    def notify_description_changed(self, event):
        pub.sendMessage(
            "model.modified.description", value=self.description_field.Value
        )

    def on_model_identifier_changed(self, value):
        self.model_identifier_field.ChangeValue(value)

    def on_name_changed(self, value):
        self.name_field.ChangeValue(value)

    def on_author_changed(self, value):
        self.author_field.ChangeValue(value)

    def on_description_changed(self, value):
        self.description_field.ChangeValue(value)


class FMI2CapabilitiesPanel(wx.Panel):
    def __init__(self, parent, id=-1) -> None:

        wx.Panel.__init__(self, parent=parent)

        def create_box(label, tooltip):
            box = wx.CheckBox(self, label=label)
            box.SetToolTip(tooltip)
            return box

        self.can_handle_variable_communication_step_size_box = create_box(
            "variable step-size", "supports calls to fmi2DoStep step length",
        )

        self.can_be_instantiated_only_once_per_process_box = create_box(
            "multiple slave instances",
            "supports the creation of several slaves by calling fmi2Instantiate multiple times",
        )

        self.can_interpolate_inputs_box = create_box("input interpolation", "todo")

        self.can_run_asynchronously_box = create_box("async support", "todo")

        self.can_not_use_memory_management_functions_box = create_box(
            "can use memory management function",
            "supports the use of tool defined functions for allocating and de-allocating memory",
        )
        self.can_get_and_set_fmu_state_box = create_box("get/set state", "todo")
        self.can_serialize_fmu_state_box = create_box(
            "serialization",
            "supports serialization of the FMUs state, allowing a snapshot of the FMU to be captured and resumed at a later stage",
        )
        self.provides_directional_derivatives_box = create_box(
            "directional derivatives",
            "provides change of the outputs as a reuslt of a change of inputs corresponding to moving along a line in the statespace",
        )

        derivatives_sizer = wx.BoxSizer()
        derivatives_label = wx.StaticText(self, label="max derivative order")
        self.derivatives_spin = wx.SpinCtrl(self)
        self.derivatives_spin.SetToolTip(
            "the highest order of the directional derivatives provided by the FMU"
        )
        derivatives_sizer.Add(self.derivatives_spin)
        derivatives_sizer.Add(derivatives_label)

        sizer = wx.FlexGridSizer(cols=3)
        sizer.Add(self.can_handle_variable_communication_step_size_box)
        sizer.Add(self.can_be_instantiated_only_once_per_process_box)
        sizer.Add(self.can_interpolate_inputs_box)
        sizer.Add(self.can_run_asynchronously_box)
        sizer.Add(self.can_not_use_memory_management_functions_box)
        sizer.Add(self.can_get_and_set_fmu_state_box)
        sizer.Add(self.can_serialize_fmu_state_box)
        sizer.Add(self.provides_directional_derivatives_box)
        sizer.Add(derivatives_sizer)
        self.SetSizer(sizer)


class ModelDescriptionPreviewPanel(wx.Panel):
    """ Panel that shows a live-preview of the model description
    """

    def __init__(self, parent) -> None:

        wx.Panel.__init__(self, parent=parent)

        sizer = BoxSizer()
        # self.md = None
        self.preview_text = wx.TextCtrl(
            self,
            -1,
            "COMING SOON!",
            wx.DefaultPosition,
            wx.Size(200, 150),
            wx.NO_BORDER
            | wx.EXPAND
            | wx.TE_READONLY
            | wx.TE_MULTILINE
            | wx.TE_DONTWRAP,
        )

        sizer.Add(self.preview_text, 1, wx.EXPAND)
        self.SetSizer(sizer)

    def set_preview(self, model: ModelDescription):
        xml = export_model_description(model).decode("utf-8")
        self.preview_text.SetValue(xml)


########################################################################
class TabPanel(wx.Panel):
    # ----------------------------------------------------------------------
    def __init__(self, parent):
        """"""
        wx.Panel.__init__(self, parent=parent)

        colors = ["red", "blue", "gray", "yellow", "green"]
        self.SetBackgroundColour(random.choice(colors))

        btn = wx.Button(self, label="Press Me")
        sizer = wx.BoxSizer(wx.VERTICAL)
        sizer.Add(btn, 0, wx.ALL, 10)
        self.SetSizer(sizer)


class HomeScreenFrame(wx.Frame):
    """
    Main frame of the application that allows the user to modify and create FMUs.

    The class acts as the controller in a MVC pattern. 
    Specifically it provides an interface which the view can use to set attributes of the model (concretely an in memory representation of the model description).
    In addition to providing access, the controller determines the visibility and "enabledness" of its children frames.
    """

    def __init__(
        self,
        parent,
        id=-1,
        title="FMU Builder",
        pos=wx.DefaultPosition,
        size=(800, 600),
        style=wx.DEFAULT_FRAME_STYLE,
    ):

        wx.Frame.__init__(self, parent, id, title, pos, size, style)
        self.mgr = aui.AuiManager()
        self.mgr.SetManagedWindow(self)

        self.live_preview_active = True
        self.model_description_preview = ModelDescriptionPreviewPanel(self)
        self.model = None
        # create a panel in the frame

        edit_panel = wx.Panel(self)
        treebook = wx.Treebook(edit_panel)
        treebook.AddPage(FMI2BasicPanel(treebook), "Info")
        treebook.AddPage(FMI2CapabilitiesPanel(treebook), "Capabilities")

        treebook.AddPage(None, "Variables")
        treebook.AddSubPage(TabPanel(treebook), "Inputs")
        treebook.AddSubPage(TabPanel(treebook), "Outputs")
        treebook.AddSubPage(TabPanel(treebook), "Parameters")

        sizer = wx.BoxSizer(wx.VERTICAL)
        sizer.Add(treebook, 1, wx.ALL | wx.EXPAND, 5)
        edit_panel.SetSizerAndFit(sizer)
        edit_panel.Hide()

        # signals
        for func, subj in [
            (self.on_author_changed, "model.modified.author"),
            (self.on_model_identifier_changed, "model.modified.model_identifier"),
            (self.on_model_name_changed, "model.modified.model_name"),
            (self.on_description_changed, "model.modified.description"),
        ]:
            pub.subscribe(func, subj)

        # pub.sendMessage("md.author", value="johnny")

        # create a menu bar
        self.makeMenuBar()

        # and a status bar
        self.CreateStatusBar()
        self.SetStatusText("Waiting for user to open an FMU")

        # dockable panels
        self.mgr.AddPane(
            self.model_description_preview,
            aui.AuiPaneInfo().Right().Caption("model description preview"),
        )
        self.mgr.AddPane(
            edit_panel, aui.AuiPaneInfo().CenterPane(),
        )
        self.mgr.Update()
        edit_panel.Disable()
        self.model_description_preview.Disable()
        self.edit_panel = edit_panel

        self.Bind(wx.EVT_CLOSE, self.on_close)

    def set_model(self, model: ModelDescription):
        self.model = model
        pub.sendMessage("model.set", model=model)
        pub.sendMessage("model.modified.author", value=model.author)
        pub.sendMessage("model.modified.model_name", value=model.model_name)
        pub.sendMessage(
            "model.modified.model_identifier",
            value=model.co_simulation.model_identifier,
        )
        pub.sendMessage(
            "model.modified.model_identifier",
            value=model.co_simulation.model_identifier,
        )

        self.model_description_preview.Enable()
        self.edit_panel.Enable()

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
        create_item = fileMenu.Append(
            -1, "&Create FMU...\tCtrl-N", "Generate a new FMU",
        )

        fileMenu.AppendSeparator()

        open_fmu_dir_archive = fileMenu.Append(
            -1, "&Open FMU archive...\tCtrl-O", "Open compressed FMU",
        )

        open_fmu_dir_item = fileMenu.Append(
            -1, "&Open FMU directory...\tCtrl+Shift-O", "Open uncompressed FMU",
        )

        fileMenu.AppendSeparator()

        # When using a stock ID we don't need to specify the menu item's
        # label
        exit_item = fileMenu.Append(wx.ID_EXIT)

        # Now a help menu for the about item
        helpMenu = wx.Menu()
        about_item = helpMenu.Append(wx.ID_ABOUT)

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
        self.Bind(wx.EVT_MENU, self.on_create, create_item)
        self.Bind(wx.EVT_MENU, self.on_edit_fmu_archive, open_fmu_dir_archive)
        self.Bind(wx.EVT_MENU, self.on_edit_fmu_directory, open_fmu_dir_item)
        self.Bind(wx.EVT_MENU, self.on_exit, exit_item)
        self.Bind(wx.EVT_MENU, self.on_about, about_item)

    def on_exit(self, event):
        """Close the frame, terminating the application."""
        self.Close(True)

    def on_close(self, event):
        # deinitialize the frame manager
        self.mgr.UnInit()
        event.Skip()

    def on_create(self, event):
        """Say hello to the user."""

        frame = CreateFMUFrame(None)
        frame.Show()

    def on_edit_fmu_archive(self, event):
        with wx.FileDialog(
            self,
            "Select a FMU archive to open",
            wildcard="fmu archives (*.fmu)|*.fmu|all files|*",
            style=wx.FD_OPEN | wx.FD_FILE_MUST_EXIST,
        ) as fileDialog:

            if fileDialog.ShowModal() != wx.ID_CANCEL:
                try:
                    self.set_model(import_fmu(fileDialog.GetPath()))
                except Exception as e:
                    wx.MessageBox(f"Unable to open FMU: {e}", style=wx.ICON_ERROR)
                    raise e

    def on_edit_fmu_directory(self, event):

        with wx.DirDialog(
            self, "Select a FMU directory to open", style=wx.DD_DIR_MUST_EXIST
        ) as dirDialog:

            if dirDialog.ShowModal() != wx.ID_CANCEL:
                try:
                    self.set_model(import_fmu(dirDialog.GetPath()))
                except Exception as e:
                    wx.MessageBox(f"Unable to open FMU: {e}", style=wx.ICON_ERROR)
                    raise e

    def on_about(self, event):
        """Display an About Dialog"""
        wx.MessageBox(
            "This is UniFMU-builder a gui tool for creating and editing FMUs:\nhttps://github.com/clegaard/unifmu",
            "UniFMU-builder",
            wx.OK | wx.ICON_INFORMATION,
        )

    def on_author_changed(self, value: str):
        self.model.author = value
        self.update_preview()

    def on_model_identifier_changed(self, value: str):
        self.model.co_simulation.model_identifier = value
        self.update_preview()

    def on_description_changed(self, value: str):
        self.model.description = value
        self.update_preview()

    def on_model_name_changed(self, value: str):
        self.model.model_name = value
        self.update_preview()

    def update_preview(self):
        """Updatet the xml preview to relect the current state of the model"""
        self.model_description_preview.set_preview(self.model)


def show_gui():
    """Show a gui that can be used to create FMUs and modify existing FMU's model description in a user friendly manner.
    The gui is build on wxpython, a cross-platform gui library that uses native widgets.

    Note that this the function that serves as a entrypoint when invoking "unifmu gui", see tool/unifmu/cli.py and setup.py.

    The gui implements an MVC pattern built on top of PyPubSub:
    https://pypubsub.readthedocs.io/en/latest/index.html
    """

    # declare topics to ease debugging, this avoids runtime inference of topic types
    # https://pypubsub.readthedocs.io/en/v4.0.3/usage/usage_advanced_maintain.html
    def on_model_set(model: ModelDescription):
        """ signal that the model has been set, e.g. a new FMU has been loaded"""
        pass

    def on_model_modified(value: int):
        """ signal that an attribute of the model has been modified
        """
        pass

    pub.subscribe(on_model_set, "model.set")
    pub.subscribe(on_model_modified, "model.modified")

    app = wx.App(0)

    frame = HomeScreenFrame(None)

    icon = wx.ArtProvider.GetIcon(wx.ART_REPORT_VIEW)
    frame.SetIcon(icon)

    app.SetTopWindow(frame)
    frame.Show()

    from pubsub.utils import printTreeDocs

    def snoop(topicObj=pub.AUTO_TOPIC, **mesgData):
        print(f'topic "{topicObj.getName()}": {mesgData}')

    pub.subscribe(snoop, pub.ALL_TOPICS)
    printTreeDocs()

    app.MainLoop()


if __name__ == "__main__":
    show_gui()
