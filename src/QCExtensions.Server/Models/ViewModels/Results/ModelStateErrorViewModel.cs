using System.Collections.Generic;
using System.Linq;
using Microsoft.AspNetCore.Mvc.ModelBinding;

namespace QCExtensions.Server.Models.ViewModels.Results
{
	public class ModelStateErrorViewModel : ResultViewModelBase
	{
		public ModelStateErrorViewModel(ModelStateDictionary errors)
		{
			foreach (var error in errors)
			{
				var errorData = new AssociatedError(error.Key, error.Value.Errors.Select(e => e.ErrorMessage).ToArray());
				Errors.Add(errorData);
			}
		}
	}
}
