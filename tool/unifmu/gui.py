#!/usr/bin/env python
"""
Hello World, but with more meat.
"""

import datetime
from os import close, spawnl
from pathlib import Path
from sys import flags
import wx
import wx.adv
import wx.gizmos

from wx.core import NumberEntryDialog, PrintDialog, VERTICAL

from unifmu.fmi2 import ModelDescription, CoSimulation


class FMI2Tooltips:

    model_identifier = "name of the shared/dynamic library located loaded by the simulation environment. The library must be located in the binaries directory of the FMU"
    model_name = (
        "human readable name of the model, typically displayed by the envrionment"
    )
    author = "creator(s) of the FMU, may be displayerd by the envrionment"
    needs_execution_tool = "Flag used to indicate that the FMU depends on at least one external tool to function correctly. The name of the tools can be retrieved from the 'generationTool' attribute"
    can_handle_variable_communication_step_size = (
        "If set, indicate that FMU supports time-steps of varying lengths"
    )
    can_be_instantiated_only_once_per_process = "Flag used to indicate that the shared library does NOT support instantiating multiple slaves"


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
        self.description_field = wx.TextCtrl(panel, style=wx.TE_RICH2 | wx.TE_MULTILINE)
        self.description_field.SetToolTip("A guide describing how to use the FMU")

        self.fmi_selector = wx.RadioBox(
            panel,
            label="FMI version",
            choices=["1.0", "2.0", "3.0"],
            majorDimension=1,
            style=wx.RA_SPECIFY_ROWS,
        )
        self.fmi_selector.SetSelection(1)
        self.fmi_selector.SetToolTip(
            "Which version of the Functional Mock-Up interface the FMU targets"
        )

        backend_label = wx.StaticText(panel, label="Backend")
        self.backend_combo = wx.ComboBox(
            panel,
            value="None",
            choices=["None", "UniFMU->Python"],
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

        name_sizer = wx.BoxSizer()
        name_sizer.Add(name_label, 0, wx.ALL, border_size)
        name_sizer.Add(self.name_field, 1, wx.ALL, border_size)

        author_sizer = wx.BoxSizer()
        author_sizer.Add(author_label, 0, wx.ALL, border_size)
        author_sizer.Add(self.author_field, 1, wx.ALL, border_size)

        description_sizer = wx.GridBagSizer()
        description_sizer.Add(
            description_label, (0, 0), (1, 1), wx.ALL | wx.EXPAND, border_size
        )
        description_sizer.Add(
            self.description_field, (0, 1), (1, 3), wx.ALL | wx.EXPAND, border_size,
        )
        description_sizer.AddGrowableRow(0)
        description_sizer.AddGrowableCol(1)

        border_size = 5
        fmi_sizer = wx.BoxSizer()
        fmi_sizer.Add(self.fmi_selector, 1, wx.ALL, border_size)
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
        main_sizer.Add(name_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(author_sizer, 0, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(description_sizer, 1, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(fmi_sizer, 0, wx.ALL | wx.EXPAND | wx.CENTER, border_size)
        main_sizer.Add(export_sizer, 1, wx.ALL | wx.EXPAND, border_size)
        main_sizer.Add(button_sizer, 0, wx.ALL | wx.CENTER, border_size)

        # fit controls
        panel.SetSizerAndFit(main_sizer)
        self.Fit()
        # self.Show()

    def on_generate(self, event):

        # --------------------- inference and defaults for missing information ----------------------

        guid = "a"
        version = "0.0.1"
        copyright = ""
        license = ""
        copyright = ""

        generation_tool = "unifmu"
        variable_naming_convention = "flat"

        # mapping is used to define defaults for variables which can be inferred from the choice of backend
        # for example unifmu variants can instantiate multiple processes, but struggle to use memory management functions
        (
            model_identifier,
            needs_execution_tool,
            can_handle_variable_communication_step_size,
            can_interpolate_inputs,
            max_output_derivative_order,
            can_run_asynchronously,
            can_be_instantiated_only_once_per_process,
            can_not_use_memory_management_functions,
            can_get_and_set_fmu_state,
            can_serialize_fmu_state,
            provides_directional_derivatives,
        ) = {
            "None": (
                "TODO",
                False,
                False,
                False,
                0,
                False,
                True,
                True,
                False,
                False,
                False,
            ),
            "UniFMU->Python": (
                "unifmu",
                True,
                True,
                False,
                0,
                False,
                False,
                True,
                False,
                False,
                False,
            ),
        }[
            self.backend_combo.Value
        ]

        # generation date and time must be some kind of xsd string
        data_time_obj = datetime.datetime.now()
        generation_date_and_time = datetime.datetime.strftime(
            data_time_obj, "%Y-%m-%dT%H:%M:%SZ"
        )

        # ------------------------ end of inference ----------------------------

        mdd = ModelDescription(
            fmi_version=self.fmi_selector.GetSelection,
            model_name=self.name_field.Value,
            guid=guid,
            description=self.description_field.Value,
            author=self.author_field,
            version=version,
            copyright=copyright,
            license=license,
            generation_tool=generation_tool,
            variable_naming_convention=variable_naming_convention,
            generation_date_and_time=generation_date_and_time,
            model_variables=[],
            model_structure=[],
            co_simulation=CoSimulation(
                model_identifier=model_identifier,
                needs_execution_tool=needs_execution_tool,
                can_handle_variable_communication_step_size=can_handle_variable_communication_step_size,
                can_interpolate_inputs=can_interpolate_inputs,
                max_output_derivative_order=max_output_derivative_order,
                can_run_asynchronously=can_run_asynchronously,
                can_be_instantiated_only_once_per_process=can_be_instantiated_only_once_per_process,
                can_not_use_memory_management_functions=can_not_use_memory_management_functions,
                can_get_and_set_fmu_state=can_get_and_set_fmu_state,
                can_serialize_fmu_state=can_serialize_fmu_state,
                provides_directional_derivatives=provides_directional_derivatives,
            ),
            model_exchange=None,
            unit_definitions=None,
            type_defintions=None,
            log_categories=[],
            default_experiment=None,
            vendor_annotations=None,
        )

        from unifmu.generate import generate_fmu

        output_path = Path(self.outdir_picker.Path) / self.filename_field.Value

        generate_fmu(
            mdd,
            path=output_path,
            backend=self.backend_combo.Value,
            zipped=self.export_zipped_box.Value,
        )

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
        panel = wx.Panel(self)

        # and create a sizer to manage the layout of child widgets

        # fields
        # --------------- fields.basic -------------------------
        self.author_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.author_field.SetToolTip(FMI2Tooltips.author)
        author_label = wx.StaticText(panel, label="Author")

        self.model_identifier_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.model_identifier_field.SetToolTip(FMI2Tooltips.model_identifier)
        model_identifier_label = wx.StaticText(panel, label="Model Identifier")

        self.name_field = wx.TextCtrl(panel, style=wx.TE_RICH2)
        self.name_field.SetToolTip(FMI2Tooltips.model_name)
        name_label = wx.StaticText(panel, label="Name")

        # ---------------- fields.capabilities ---------------------

        def create_box(label, tooltip):
            box = wx.CheckBox(panel, label=label)
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
        derivatives_label = wx.StaticText(panel, label="max derivative order")
        self.derivatives_spin = wx.SpinCtrl(panel)
        self.derivatives_spin.SetToolTip(
            "the highest order of the directional derivatives provided by the FMU"
        )
        derivatives_sizer.Add(self.derivatives_spin)
        derivatives_sizer.Add(derivatives_label)

        # --------------- fields.scalar_variables ------------------------

        # sizers

        basic_sizer = wx.FlexGridSizer(rows=3, cols=2, hgap=5, vgap=5)
        basic_sizer.AddGrowableCol(1)

        basic_sizer.Add(author_label)
        basic_sizer.Add(self.author_field, flag=wx.EXPAND)

        basic_sizer.Add(model_identifier_label)
        basic_sizer.Add(self.model_identifier_field, flag=wx.EXPAND)

        basic_sizer.Add(name_label)
        basic_sizer.Add(self.name_field, flag=wx.EXPAND)

        capabilities_sizer = wx.FlexGridSizer(cols=3)
        capabilities_sizer.Add(self.can_handle_variable_communication_step_size_box)
        capabilities_sizer.Add(self.can_be_instantiated_only_once_per_process_box)
        capabilities_sizer.Add(self.can_interpolate_inputs_box)
        capabilities_sizer.Add(self.can_run_asynchronously_box)
        capabilities_sizer.Add(self.can_not_use_memory_management_functions_box)
        capabilities_sizer.Add(self.can_get_and_set_fmu_state_box)
        capabilities_sizer.Add(self.can_serialize_fmu_state_box)
        capabilities_sizer.Add(self.provides_directional_derivatives_box)
        capabilities_sizer.Add(derivatives_sizer)

        main_sizer = wx.BoxSizer(wx.VERTICAL)
        main_sizer.Add(wx.StaticText(panel, label="Basic"), flag=wx.CENTER)
        main_sizer.Add(wx.StaticLine(panel), flag=wx.EXPAND)
        main_sizer.Add(basic_sizer, flag=wx.EXPAND)

        main_sizer.Add(wx.StaticText(panel, label="Capabilities"), flag=wx.CENTER)
        main_sizer.Add(wx.StaticLine(panel), flag=wx.EXPAND)
        main_sizer.Add(capabilities_sizer, 1, flag=wx.EXPAND)

        # create a menu bar
        self.makeMenuBar()

        # and a status bar
        self.CreateStatusBar()
        self.SetStatusText("Waiting for user")

        panel.SetSizer(main_sizer)
        main_sizer.Fit(self)

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
            -1, "&Create FMU...\tCtrl-H", "Generate a template for a new FMU",
        )
        fileMenu.AppendSeparator()

        edit_item = fileMenu.Append(-1, "&Edit FMU...\tCtrl-H", "Modify existing FMU",)

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
        self.Bind(wx.EVT_MENU, self.on_edit, edit_item)
        self.Bind(wx.EVT_MENU, self.on_exit, exit_item)
        self.Bind(wx.EVT_MENU, self.on_about, about_item)

    def on_exit(self, event):
        """Close the frame, terminating the application."""
        self.Close(True)

    def on_create(self, event):
        """Say hello to the user."""

        frame = CreateFMUFrame(None)
        frame.Show()

        # dummy data
        global md

    def on_edit(self, event):

        with wx.DirDialog(
            self, "Select a directory to place the generated FMU inside"
        ) as dirDialog:

            if dirDialog.ShowModal() != wx.ID_CANCEL:
                pathname = dirDialog.GetPath()
                print(f"path is {pathname}")

    def on_about(self, event):
        """Display an About Dialog"""
        wx.MessageBox(
            "This is UniFMU-builder a gui tool for creating and editing FMUs:\nhttps://github.com/clegaard/unifmu",
            "UniFMU-builder",
            wx.OK | wx.ICON_INFORMATION,
        )


md = None


def show_gui():
    app = wx.App()
    frm = HomeScreenFrame(title="FMU Builder")
    frm.Show()

    app.MainLoop()

