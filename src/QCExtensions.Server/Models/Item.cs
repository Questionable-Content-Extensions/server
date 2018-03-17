using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;
using Newtonsoft.Json;

namespace QCExtensions.Server.Models
{
	[Table("items")]
	public class Item
	{
		[Key]
		[Column("id")]
		public int Id { get; set; }

		[Column("shortName")]
		public string ShortName { get; set; }

		[Column("name")]
		public string Name { get; set; }

		[Column("type")]
		public string Type
		{
			get
			{
				return TypeValue.ToString().ToLower();
			}
			set
			{
				switch (value)
				{
					case "cast":
						TypeValue = ItemType.Cast;
						break;

					case "location":
						TypeValue = ItemType.Location;
						break;

					case "storyline":
						TypeValue = ItemType.Storyline;
						break;

					default:
						TypeValue = ItemType.Unknown;
						break;
				}
			}
		}
		[NotMapped]
		[JsonIgnore]
		public ItemType TypeValue { get; set; }

		[Column("color")]
		public string Color { get; set; }

		[JsonIgnore]
		public ICollection<Occurrences> Occurrences { get; set; }
	}
}