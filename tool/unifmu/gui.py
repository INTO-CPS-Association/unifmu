from os import mkdir, name
from pathlib import Path
from sys import settrace
from typing import List
from pubsub.pub import validate

import wx
import wx.adv
from wx.core import BoxSizer, FlexGridSizer, GridSizer
import wx.lib.agw.aui as aui
import wx.gizmos
from pubsub import pub


from unifmu.fmi2 import (
    ModelDescription,
    CoSimulation,
    ScalarVariable,
    get_causality_to_variability_choices,
    get_intitial_choices_and_default,
    get_should_define_start,
)
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
            wx.EVT_TEXT,
            lambda _: pub.sendMessage(
                "model.attr_modified",
                key="modelIdentifier",
                value=self.model_identifier_field.Value,
            ),
        )
        self.name_field.Bind(
            wx.EVT_TEXT,
            lambda _: pub.sendMessage(
                "model.attr_modified", key="modelName", value=self.name_field.Value
            ),
        )
        self.author_field.Bind(
            wx.EVT_TEXT,
            lambda _: pub.sendMessage(
                "model.attr_modified", key="author", value=self.author_field.Value
            ),
        )
        self.description_field.Bind(
            wx.EVT_TEXT,
            lambda _: pub.sendMessage(
                "model.attr_modified",
                key="description",
                value=self.description_field.Value,
            ),
        )

        pub.subscribe(self.on_model_attr_modified, "model.attr_modified")

    def on_model_attr_modified(self, key, value, sender=None):
        if sender is self:
            return

        key_to_field = {
            "modelName": self.name_field,
            "modelIdentifier": self.model_identifier_field,
            "description": self.description_field,
            "author": self.author_field,
        }

        if key in key_to_field:
            key_to_field[key].ChangeValue(value)


class FMI2CapabilitiesPanel(wx.Panel):
    def __init__(self, parent, id=-1) -> None:

        wx.Panel.__init__(self, parent=parent)

        def create_box(label, tooltip):
            box = wx.CheckBox(self, label=label)
            box.SetToolTip(tooltip)
            return box

        self.canHandleVariableCommunicationStepSizeBox = create_box(
            "variable step-size", "supports calls to fmi2DoStep step length",
        )

        self.canBeInstantiatedOnlyOncePerProcessBox = create_box(
            "multiple slave instances",
            "supports the creation of several slaves by calling fmi2Instantiate multiple times",
        )

        self.canInterpolateInputsBox = create_box("input interpolation", "todo")

        self.canRunAsynchronuouslyBox = create_box("async support", "todo")

        self.canNotUseMemoryManagementFunctionsBox = create_box(
            "can use memory management function",
            "supports the use of tool defined functions for allocating and de-allocating memory",
        )
        self.canGetAndSetFmuStateBox = create_box("get/set state", "todo")
        self.canSerializeFmuStateBox = create_box(
            "serialization",
            "supports serialization of the FMUs state, allowing a snapshot of the FMU to be captured and resumed at a later stage",
        )
        self.providesDirectionalDerivativeBox = create_box(
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
        sizer.Add(self.canHandleVariableCommunicationStepSizeBox)
        sizer.Add(self.canBeInstantiatedOnlyOncePerProcessBox)
        sizer.Add(self.canInterpolateInputsBox)
        sizer.Add(self.canRunAsynchronuouslyBox)
        sizer.Add(self.canNotUseMemoryManagementFunctionsBox)
        sizer.Add(self.canGetAndSetFmuStateBox)
        sizer.Add(self.canSerializeFmuStateBox)
        sizer.Add(self.providesDirectionalDerivativeBox)
        sizer.Add(derivatives_sizer)
        self.SetSizer(sizer)

        def bind_box(box, attr_name: str):
            box.Bind(
                wx.EVT_CHECKBOX,
                lambda _: pub.sendMessage(
                    "model.attr_modified", key=attr_name, value=box.Value, sender=self
                ),
            )

        for box, attr_name in [
            (
                self.canHandleVariableCommunicationStepSizeBox,
                "canHandleVariableCommunicationStepSize",
            ),
            (
                self.canBeInstantiatedOnlyOncePerProcessBox,
                "canBeInstantiatedOnlyOncePerProcess",
            ),
            (self.canInterpolateInputsBox, "canInterpolateInputs",),
            (self.canRunAsynchronuouslyBox, "canRunAsynchronuously",),
            (
                self.canNotUseMemoryManagementFunctionsBox,
                "canNotUseMemoryManagementFunctions",
            ),
            (self.canGetAndSetFmuStateBox, "canGetAndSetFMUstate",),
            (self.canSerializeFmuStateBox, "canSerializeFMUstate",),
            (self.providesDirectionalDerivativeBox, "providesDirectionalDerivative",),
        ]:
            bind_box(box, attr_name)


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


class FMI2VariableEditorPanel(wx.Panel):
    """ Panel that allows editing of all types of scalar variables, e.g. inputs, outputs, parameters, etc.

        Based on the variables causality the panel restricts the combinations to include only those that are
        valid for the particular choice of causality.
    """

    def __init__(self, parent, variable: ScalarVariable) -> None:

        wx.Panel.__init__(self, parent=parent)

        sizer = BoxSizer()
        self.SetSizerAndFit(sizer)

        self.variable = variable

        sizer = FlexGridSizer(2)

        # data type
        self.type_combo = wx.ComboBox(
            self,
            choices=["Real", "Integer", "Boolean", "String"],
            value=self.variable.dataType,
        )
        sizer.Add(wx.StaticText(self, label="Data-Type:"))
        sizer.Add(self.type_combo)

        # variability
        self.variability_combo = wx.ComboBox(
            self,
            choices=get_causality_to_variability_choices(self.variable.causality),
            value=self.variable.variability,
        )
        sizer.Add(wx.StaticText(self, label="Variability:"))
        sizer.Add(self.variability_combo)

        # initial

        self.initial_label = wx.StaticText(self, label="Initial:")
        self.initial_combo = wx.ComboBox(self)
        sizer.Add(self.initial_label)
        sizer.Add(self.initial_combo)

        # start
        self.start_label = wx.StaticText(self, label="Start:")
        self.start_field = wx.TextCtrl(self)
        sizer.Add(self.start_label)
        sizer.Add(self.start_field)

        # min
        self.min_field = wx.TextCtrl(self)
        self.min_label = wx.StaticText(self, label="Min:")
        sizer.Add(self.min_label)
        sizer.Add(self.min_field)

        # max
        self.max_field = wx.TextCtrl(self)
        self.max_label = wx.StaticText(self, label="Max:")
        sizer.Add(self.max_label)
        sizer.Add(self.max_field)

        # nominal:
        self.nominal_label = wx.StaticText(self, label="Nominal:")
        self.nominal_field = wx.TextCtrl(self)
        sizer.Add(self.nominal_label)
        sizer.Add(self.nominal_field)

        # description
        description_field = wx.TextCtrl(self, value=self.variable.description)
        sizer.Add(wx.StaticText(self, label="Description:"))
        sizer.Add(description_field)

        self.SetSizer(sizer)

        # events

        self.Bind(
            wx.EVT_COMBOBOX, self.on_controls_modified,
        )

        pub.subscribe(self.on_variable_modified, "scalar_variable.modified")
        self._update_choices()

    def on_controls_modified(self, _):

        self.variable.initial = self.initial_combo.Value
        self.variable.dataType = self.type_combo.Value
        self.variable.variability = self.variability_combo.Value
        # self.variable.nominal = self.initial_combo.Value

        pub.sendMessage("scalar_variable.modified", variable=self.variable, sender=self)

    def _update_choices(self):
        """Update the possible choices of the controls and hide fields that should not be accessed"""

        def update_choices(combo, choices):
            combo.Clear()
            for c in choices:
                combo.Append(c)

        # initial
        _, initial_choices = get_intitial_choices_and_default(
            self.variable.causality, variability=self.variable.variability
        )
        update_choices(self.initial_combo, initial_choices)

        if initial_choices != []:
            self.initial_combo.ChangeValue(self.variable.initial)
            self.initial_label.Show()
            self.initial_combo.Show()
        else:
            self.initial_label.Hide()
            self.initial_combo.Hide()

        # start
        if get_should_define_start(self.variable.initial):
            self.start_field.ChangeValue(self.variable.start)
            self.start_label.Show()
            self.start_field.Show()

        else:
            self.start_label.Hide()
            self.start_field.Hide()

        # nominal, min, max
        if self.variable.dataType != "Real":

            self.nominal_label.Hide()
            self.nominal_field.Hide()
            self.min_label.Hide()
            self.min_field.Hide()
            self.max_label.Hide()
            self.max_field.Hide()
        else:
            self.nominal_label.Show()
            self.nominal_field.Show()
            self.min_label.Show()
            self.min_field.Show()
            self.max_label.Show()
            self.max_field.Show()

        self.Fit()

    def on_variable_modified(self, variable: ScalarVariable, sender=None):

        if sender is self or variable.name != self.variable.name:
            return

        self.variable = variable
        self._update_choices()


class FMI2VariableBook(wx.Panel):
    def __init__(self, parent, causality: str) -> None:

        wx.Panel.__init__(self, parent=parent)
        self.causality = causality
        self.book = wx.Listbook(self, style=wx.LC_EDIT_LABELS, name=causality)

        # you should never see the page before select

        sizer = BoxSizer()
        sizer.Add(self.book, 1, wx.ALL | wx.EXPAND, 5)
        self.SetSizerAndFit(sizer)

        pub.subscribe(self.on_scalar_variable_added, "scalar_variable.added")
        pub.subscribe(self.on_scalar_variable_removed, "scalar_variable.removed")
        self.name_to_page = {}

        self.book.Bind(
            wx.EVT_LIST_BEGIN_DRAG, lambda evt: print(f"delete event: {evt}")
        )

    def on_scalar_variable_removed(self, variable: ScalarVariable, sender=None):
        if sender is self:
            return
        if variable.name in self.name_to_page:
            self.book.RemovePage(self.name_to_page[variable.name])

    def on_scalar_variable_added(self, variable: ScalarVariable, sender=None):
        if sender is self:
            return
        if variable.causality != self.causality:
            return

        self.name_to_page[variable.name] = variable
        self.book.AddPage(FMI2VariableEditorPanel(self.book, variable), variable.name)


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

        treebook.AddPage(FMI2VariableBook(treebook, "input"), "Inputs")
        treebook.AddPage(FMI2VariableBook(treebook, "output"), "Outputs")
        treebook.AddPage(FMI2VariableBook(treebook, "parameter"), "Parameters")

        sizer = wx.BoxSizer(wx.VERTICAL)
        sizer.Add(treebook, 1, wx.ALL | wx.EXPAND, 5)
        edit_panel.SetSizerAndFit(sizer)
        edit_panel.Hide()

        # bind events generated by views to the model
        pub.subscribe(self.on_model_attr_modified, "model.attr_modified")
        pub.subscribe(self.on_scalar_variable_added, "scalar_variable.added")
        pub.subscribe(self.on_scalar_variable_modified, "scalar_variable.modified")
        pub.subscribe(self.on_scalar_variable_removed, "scalar_variable.removed")

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

    def on_scalar_variable_added(self, variable: ScalarVariable, sender=None):
        if sender is self:
            return
        raise NotImplementedError()

    def on_scalar_variable_modified(self, variable: ScalarVariable, sender=None):
        if sender is self:
            return

        idx = next(
            idx
            for idx, v in enumerate(self.model.modelVariables)
            if v.name == variable.name
        )

        self.model.modelVariables[idx] = variable

        # if need start value
        if get_should_define_start(variable.initial) and variable.start is None:
            variable.start = "INFERRED BY controller"

        pub.sendMessage("scalar_variable.modified", variable=variable, sender=self)

        self.model_description_preview.set_preview(self.model)

    def on_scalar_variable_removed(self, variable: ScalarVariable, sender=None):
        if sender is self:
            return
        raise NotImplementedError()

    def set_model(self, model: ModelDescription):
        self.model = model
        pub.sendMessage("model.set", model=model, sender=self)
        pub.sendMessage(
            "model.attr_modified", key="author", value=model.author, sender=self
        )
        pub.sendMessage(
            "model.attr_modified", key="modelName", value=model.modelName, sender=self
        )
        pub.sendMessage(
            "model.attr_modified",
            key="modelIdentifier",
            value=model.CoSimulation.modelIdentifier,
            sender=self,
        )
        pub.sendMessage(
            "model.attr_modified",
            key="description",
            value=model.description,
            sender=self,
        )

        for sv in model.modelVariables:
            pub.sendMessage("scalar_variable.added", variable=sv, sender=self)

        pub.subscribe(self.on_model_attr_modified, "model.attr_modified")

        self.model_description_preview.Enable()
        self.edit_panel.Enable()
        self.model_description_preview.set_preview(self.model)

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

    def on_model_attr_modified(self, key: str, value, sender=None):

        if sender is self:
            return

        assert self.model is not None

        if key in self.model.__dict__:
            setattr(self.model, key, value)
        elif key in self.model.CoSimulation.__dict__:
            setattr(self.model.CoSimulation, key, value)
        else:
            raise KeyError(f"Unrecognized model attribute: '{key}'")

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
    def on_model_set(model: ModelDescription, sender=None):
        """ signal that the model has been set, e.g. a new FMU has been loaded"""
        pass

    def on_model_attr_modified(key: str, value, sender=None):
        """ signal that an attribute of the model has been modified
        """
        pass

    def on_scalar_variable_modified(variable: ScalarVariable, sender=None):
        """ Scalar variable modified.
        """
        pass

    def on_scalar_variable_added(variable: ScalarVariable, sender=None):
        pass

    def on_scalar_variable_removed(variable: ScalarVariable, sender=None):
        """ Scalar variable removed """
        pass

    def on_scalar_variable_renamed(old_name: str, new_name: str, sender=None):
        pass

    pub.subscribe(on_model_set, "model.set")
    pub.subscribe(on_model_attr_modified, "model.attr_modified")
    pub.subscribe(on_scalar_variable_modified, "scalar_variable.modified")
    pub.subscribe(on_scalar_variable_added, "scalar_variable.added")
    pub.subscribe(on_scalar_variable_removed, "scalar_variable.removed")
    pub.subscribe(on_scalar_variable_renamed, "scalar_variable.renamed")

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
    print("")

    app.MainLoop()


if __name__ == "__main__":
    show_gui()
